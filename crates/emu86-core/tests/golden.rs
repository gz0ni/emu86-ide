use emu86_core::{Cpu, Flags, COM_LOAD_ADDRESS};
use std::fs;
use std::path::{Path, PathBuf};

const CHECKSUM_START: usize = 0x0100;
const CHECKSUM_END: usize = 0x1_0000;
const FNV_OFFSET: u32 = 0x811C_9DC5;
const FNV_PRIME: u32 = 0x0100_0193;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct Registers {
    ax: u16,
    bx: u16,
    cx: u16,
    dx: u16,
    sp: u16,
    bp: u16,
    si: u16,
    di: u16,
    ds: u16,
    es: u16,
    ss: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Snapshot {
    ip: u16,
    cs: u16,
    regs: Registers,
    flags: u16,
    mem_checksum: u32,
    halted: bool,
}

#[test]
fn replays_committed_golden_traces() {
    let root = golden_root();
    let entries = fs::read_dir(&root)
        .unwrap_or_else(|error| panic!("cannot read {}: {error}", root.display()));
    let mut found = 0;

    for entry in entries {
        let directory = entry.expect("directory entry").path();
        if !directory.is_dir() || !directory.join("trace.jsonl").exists() {
            continue;
        }

        replay_dir(&directory).unwrap_or_else(|error| panic!("{}: {error}", directory.display()));
        found += 1;
    }

    if found == 0 {
        assert!(
            std::env::var_os("EMU86_GOLDEN_ROOT").is_none(),
            "explicit golden root contains no traces"
        );
        eprintln!("no golden traces found under {}", root.display());
    }
}

#[test]
fn synthetic_trace_replays_and_rejects_corruption() {
    let directory = unique_temp_dir();
    fs::create_dir_all(&directory).expect("create temporary directory");

    let program = [0xB8, 0x34, 0x12, 0x90];
    let mut memory = emu86_core::Memory::new();
    memory.load_linear(usize::from(COM_LOAD_ADDRESS), &program);
    let checksum = checksum(memory.as_slice());

    let initial = snapshot_json(
        0x0100,
        0,
        Registers {
            sp: 0xFFFE,
            ..Registers::default()
        },
        0xF002,
        checksum,
        false,
    );
    let before_nop = snapshot_json(
        0x0103,
        0,
        Registers {
            ax: 0x1234,
            sp: 0xFFFE,
            ..Registers::default()
        },
        0xF002,
        checksum,
        false,
    );
    let final_state = snapshot_json(
        0x0104,
        0,
        Registers {
            ax: 0x1234,
            sp: 0xFFFE,
            ..Registers::default()
        },
        0xF002,
        checksum,
        false,
    );

    fs::write(directory.join("program.com"), program).expect("write program");
    fs::write(
        directory.join("trace.jsonl"),
        format!("{initial}\n{before_nop}\n"),
    )
    .expect("write trace");
    fs::write(directory.join("final.json"), &final_state).expect("write final state");

    replay_dir(&directory).expect("synthetic trace should replay");

    fs::write(
        directory.join("final.json"),
        final_state.replace("\"ax\":4660", "\"ax\":4661"),
    )
    .expect("write corrupted final state");
    assert!(replay_dir(&directory).is_err());

    fs::remove_dir_all(directory).expect("remove temporary directory");
}

fn golden_root() -> PathBuf {
    std::env::var_os("EMU86_GOLDEN_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/golden"))
}

#[test]
fn snapshot_parser_accepts_json_whitespace() {
    let compact = snapshot_json(256, 0, Registers::default(), 0xF002, 0, false);
    let spaced = compact.replace(':', " \n : \t ").replace(',', ",\r\n");
    assert_eq!(parse_snapshot(&spaced), parse_snapshot(&compact));
}

#[test]
fn snapshot_parser_rejects_malformed_or_ambiguous_states() {
    let valid = snapshot_json(256, 0, Registers::default(), 0xF002, 0, false);
    for invalid in [
        valid.replace("\"ip\":256", "\"ip\":256,\"ip\":257"),
        valid.replace("\"ip\":256", "\"ip\":65536"),
        valid.replace("\"ip\":256", "\"ip\":null"),
        valid.replace("\"ip\":256", "\"ip\":\"256\""),
        valid.replace("\"ip\":256", "\"ip\":256.1"),
        valid.replace("\"ip\":256", "\"ip\":0256"),
        valid.replace("\"ip\":256", "\"ip\":+256"),
        valid.replace("\"ip\":256,", ""),
        valid.replace("\"ax\":0", "\"ax\":0,\"ax\":1"),
        valid.replace("\"ax\":0,", ""),
        valid.replace("\"cs\":0,", "\"regs\":{\"cs\":0},"),
        valid.replace("\"halted\":false", "\"halted\":false,\"unknown\":1"),
        valid.replace("\"halted\":false", "\"halted\":false,"),
        format!("{{\"ignored\":{valid}}}"),
        format!("{valid}{valid}"),
        format!("{valid}\u{00A0}"),
    ] {
        assert!(parse_snapshot(&invalid).is_err(), "accepted {invalid}");
    }
}

#[test]
fn replay_checks_halt_sequence_and_reports_instruction_errors() {
    let mut memory = emu86_core::Memory::new();
    memory.load_linear(0x0100, &[0x90, 0xF4]);
    let hash = checksum(memory.as_slice());
    let state = |ip, halted| snapshot_json(ip, 0, Registers::default(), 0xF002, hash, halted);
    let initial = state(0x0100, false);
    let before_halt = state(0x0101, false);
    let final_state = state(0x0101, true);
    assert!(replay(
        &[0x90, 0xF4],
        &format!("{initial}\n{before_halt}"),
        &final_state
    )
    .is_ok());
    assert!(replay(&[0x90, 0xF4], "", &final_state).is_err());
    assert!(replay(&vec![0; 0xFF01], &initial, &final_state).is_err());
    assert!(replay(&[0x90, 0xF4], &initial, &final_state).is_err());
    assert!(replay(
        &[0x90, 0xF4],
        &format!("{initial}\n{before_halt}\n{final_state}"),
        &final_state
    )
    .is_err());

    memory.load_linear(0x0100, &[0x60]);
    let invalid = snapshot_json(
        0x0100,
        0,
        Registers::default(),
        0xF002,
        checksum(memory.as_slice()),
        false,
    );
    let error = replay(&[0x60, 0xF4], &invalid, &final_state).unwrap_err();
    assert!(error.contains("instruction 1"), "{error}");
}

#[test]
fn replay_detects_memory_and_flag_divergence_at_the_correct_step() {
    let program = [0xB0, 0x7F, 0xA2, 0x00, 0x02, 0xF4];
    let regs = Registers::default();
    let loaded = Registers { ax: 0x7F, ..regs };
    let initial = snapshot_json(0x0100, 0, regs, 0xF002, 2650794588, false);
    let before_store = snapshot_json(0x0102, 0, loaded, 0xF002, 2650794588, false);
    let before_halt = snapshot_json(0x0105, 0, loaded, 0xF002, 2899505699, false);
    let final_state = snapshot_json(0x0105, 0, loaded, 0xF002, 2899505699, true);
    let trace = format!("{initial}\n{before_store}\n{before_halt}");
    replay(&program, &trace, &final_state).expect("memory-writing trace replays");

    let error = replay(
        &program,
        &trace.replace("2899505699", "2650794588"),
        &final_state,
    )
    .unwrap_err();
    assert!(
        error.contains("before instruction 3: mem_checksum"),
        "{error}"
    );
    let error = replay(&program, &trace, &final_state.replace("61442", "61443")).unwrap_err();
    assert!(error.contains("final state: flags"), "{error}");
}

#[test]
fn hlt_preserves_the_reference_instruction_pointer() {
    let directory =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/golden/arithmetic-overflow");
    let program = fs::read(directory.join("program.bin")).unwrap();
    let memory = fs::read(directory.join("initial-memory.bin")).unwrap();
    let mut trace = fs::read_to_string(directory.join("trace.jsonl")).unwrap();
    let before_hlt = fs::read_to_string(directory.join("final.json")).unwrap();
    trace.push_str(&before_hlt);
    let after_hlt = before_hlt.replace("\"halted\":false", "\"halted\":true");
    replay_with_memory(&program, &trace, &after_hlt, Some(&memory))
        .expect("reference HLT keeps IP on the stopping opcode");
}

#[test]
fn replay_preserves_raw_reference_flags() {
    let initial = snapshot_json(0x0100, 0, Registers::default(), 0x0202, 341274553, false);
    let final_state = snapshot_json(0x0101, 0, Registers::default(), 0x0202, 341274553, false);
    replay(&[0x90, 0xF4], &initial, &final_state).expect("raw reference flags replay");
    let error = replay(&[0x90, 0xF4], &initial, &final_state.replace("514", "515")).unwrap_err();
    assert!(error.contains("final state: flags"), "{error}");
}

#[test]
fn replay_loads_reference_memory_at_the_recorded_entry_point() {
    let directory = unique_temp_dir();
    fs::create_dir_all(&directory).expect("create temporary directory");
    let program = [0xA1, 0x00, 0x02, 0x90];
    let mut memory = vec![0; 65536];
    memory[0x0200..0x0202].copy_from_slice(&[0x34, 0x12]);
    memory[0x1100..0x1104].copy_from_slice(&program);
    fs::write(directory.join("initial-memory.bin"), memory).expect("write memory dump");
    fs::write(directory.join("program.bin"), program).expect("write program");
    let initial = snapshot_json(
        0x0100,
        0x0100,
        Registers::default(),
        0x0202,
        3977269960,
        false,
    );
    let final_state = snapshot_json(
        0x0103,
        0x0100,
        Registers {
            ax: 0x1234,
            ..Registers::default()
        },
        0x0202,
        3977269960,
        false,
    );
    fs::write(directory.join("trace.jsonl"), initial).expect("write trace");
    fs::write(directory.join("final.json"), final_state).expect("write final state");
    let result = replay_dir(&directory);
    fs::remove_dir_all(directory).expect("remove temporary directory");
    result.expect("relocated reference memory replays");
}

#[test]
fn replay_rejects_invalid_reference_memory_images() {
    let directory = unique_temp_dir();
    fs::create_dir_all(&directory).expect("create temporary directory");
    let initial = snapshot_json(0x0100, 0, Registers::default(), 0xF002, 341274553, false);
    fs::write(directory.join("program.bin"), [0x90, 0xF4]).expect("write program");
    fs::write(directory.join("trace.jsonl"), &initial).expect("write trace");
    fs::write(directory.join("final.json"), initial.replace("256", "257"))
        .expect("write final state");
    for size in [0, 65535, 65537] {
        fs::write(directory.join("initial-memory.bin"), vec![0; size]).expect("write dump");
        let error = replay_dir(&directory).unwrap_err();
        assert!(error.contains("initial memory"), "{error}");
    }
    fs::write(directory.join("initial-memory.bin"), vec![0; 65536]).expect("write dump");
    let error = replay_dir(&directory).unwrap_err();
    assert!(error.contains("program image"), "{error}");
    for (ip, cs) in [(0x00FF, 0), (0, 0x1000), (0xFFFF, 0)] {
        fs::write(
            directory.join("trace.jsonl"),
            snapshot_json(ip, cs, Registers::default(), 0xF002, 0, false),
        )
        .expect("write trace");
        let error = replay_dir(&directory).unwrap_err();
        assert!(error.contains("checksum window"), "{error}");
    }
    fs::write(directory.join("program.bin"), []).expect("write empty program");
    let error = replay_dir(&directory).unwrap_err();
    assert!(error.contains("empty"), "{error}");
    fs::remove_dir_all(directory).expect("remove temporary directory");
}

fn unique_temp_dir() -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system clock after the epoch")
        .as_nanos();
    std::env::temp_dir().join(format!("emu86-golden-{}-{nanos}", std::process::id()))
}

fn replay_dir(directory: &Path) -> Result<(), String> {
    let program_path = if directory.join("program.com").exists() {
        directory.join("program.com")
    } else {
        directory.join("program.bin")
    };
    let program = fs::read(&program_path)
        .map_err(|error| format!("cannot read {}: {error}", program_path.display()))?;
    let trace = fs::read_to_string(directory.join("trace.jsonl"))
        .map_err(|error| format!("cannot read trace.jsonl: {error}"))?;
    let final_state = fs::read_to_string(directory.join("final.json"))
        .map_err(|error| format!("cannot read final.json: {error}"))?;
    let initial_memory = match fs::read(directory.join("initial-memory.bin")) {
        Ok(memory) => Some(memory),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
        Err(error) => return Err(format!("cannot read initial-memory.bin: {error}")),
    };
    replay_with_memory(&program, &trace, &final_state, initial_memory.as_deref())
}

fn replay(program: &[u8], trace: &str, final_state: &str) -> Result<(), String> {
    replay_with_memory(program, trace, final_state, None)
}

fn replay_with_memory(
    program: &[u8],
    trace: &str,
    final_state: &str,
    initial_memory: Option<&[u8]>,
) -> Result<(), String> {
    if program.len() > CHECKSUM_END - CHECKSUM_START {
        return Err(format!(
            "program is {} bytes, but the M1 checksum window holds at most {}",
            program.len(),
            CHECKSUM_END - CHECKSUM_START
        ));
    }

    let snapshots = trace
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(parse_snapshot)
        .collect::<Result<Vec<_>, _>>()?;
    let Some(first) = snapshots.first() else {
        return Err("trace is empty".into());
    };

    if snapshots.iter().any(|snapshot| snapshot.halted) {
        return Err("trace rows must describe running states before instructions".into());
    }

    let mut cpu = Cpu::new();
    if let Some(memory) = initial_memory {
        if memory.len() != CHECKSUM_END {
            return Err("initial memory must contain exactly 65536 bytes".into());
        }
        if program.is_empty() {
            return Err("program image is empty".into());
        }
        let start = emu86_core::Memory::physical(first.cs, first.ip);
        let end = start + program.len();
        if start < CHECKSUM_START || end > CHECKSUM_END {
            return Err("program image must lie within the checksum window".into());
        }
        if memory[start..end] != *program {
            return Err("program image does not match initial memory at CS:IP".into());
        }
        cpu.mem.load_linear(0, memory);
    } else {
        cpu.mem.load_linear(usize::from(COM_LOAD_ADDRESS), program);
    }

    apply(&mut cpu, first);
    compare(&cpu, first, "initial state")?;

    let mut index = 0;
    while index + 1 < snapshots.len() {
        if cpu.halted {
            return Err(format!("machine halted before trace line {}", index + 2));
        }
        cpu.step()
            .map_err(|error| format!("instruction {}: {error}", index + 1))?;
        index += 1;
        compare(
            &cpu,
            &snapshots[index],
            &format!("before instruction {}", index + 1),
        )?;
    }

    if !cpu.halted {
        cpu.step()
            .map_err(|error| format!("instruction {}: {error}", snapshots.len()))?;
    }

    let expected = parse_snapshot(final_state)?;
    compare(&cpu, &expected, "final state")
}

fn apply(cpu: &mut Cpu, snapshot: &Snapshot) {
    cpu.regs.ip = snapshot.ip;
    cpu.regs.cs = snapshot.cs;
    cpu.regs.ax = snapshot.regs.ax;
    cpu.regs.bx = snapshot.regs.bx;
    cpu.regs.cx = snapshot.regs.cx;
    cpu.regs.dx = snapshot.regs.dx;
    cpu.regs.sp = snapshot.regs.sp;
    cpu.regs.bp = snapshot.regs.bp;
    cpu.regs.si = snapshot.regs.si;
    cpu.regs.di = snapshot.regs.di;
    cpu.regs.ds = snapshot.regs.ds;
    cpu.regs.es = snapshot.regs.es;
    cpu.regs.ss = snapshot.regs.ss;
    cpu.regs.flags = Flags(snapshot.flags);
    cpu.halted = snapshot.halted;
}

fn compare(cpu: &Cpu, expected: &Snapshot, context: &str) -> Result<(), String> {
    let actual = Snapshot {
        ip: cpu.regs.ip,
        cs: cpu.regs.cs,
        regs: Registers {
            ax: cpu.regs.ax,
            bx: cpu.regs.bx,
            cx: cpu.regs.cx,
            dx: cpu.regs.dx,
            sp: cpu.regs.sp,
            bp: cpu.regs.bp,
            si: cpu.regs.si,
            di: cpu.regs.di,
            ds: cpu.regs.ds,
            es: cpu.regs.es,
            ss: cpu.regs.ss,
        },
        flags: cpu.regs.flags.bits(),
        mem_checksum: checksum(cpu.mem.as_slice()),
        halted: cpu.halted,
    };

    if actual == *expected {
        return Ok(());
    }

    let mut differences = Vec::new();
    if actual.ip != expected.ip {
        differences.push(format!("ip {:#06X} != {:#06X}", actual.ip, expected.ip));
    }
    if actual.cs != expected.cs {
        differences.push(format!("cs {:#06X} != {:#06X}", actual.cs, expected.cs));
    }
    for (name, actual, expected) in [
        ("ax", actual.regs.ax, expected.regs.ax),
        ("bx", actual.regs.bx, expected.regs.bx),
        ("cx", actual.regs.cx, expected.regs.cx),
        ("dx", actual.regs.dx, expected.regs.dx),
        ("sp", actual.regs.sp, expected.regs.sp),
        ("bp", actual.regs.bp, expected.regs.bp),
        ("si", actual.regs.si, expected.regs.si),
        ("di", actual.regs.di, expected.regs.di),
        ("ds", actual.regs.ds, expected.regs.ds),
        ("es", actual.regs.es, expected.regs.es),
        ("ss", actual.regs.ss, expected.regs.ss),
    ] {
        if actual != expected {
            differences.push(format!("{name} {actual:#06X} != {expected:#06X}"));
        }
    }
    if actual.flags != expected.flags {
        differences.push(format!(
            "flags {:#06X} != {:#06X}",
            actual.flags, expected.flags
        ));
    }
    if actual.mem_checksum != expected.mem_checksum {
        differences.push(format!(
            "mem_checksum {:#010X} != {:#010X}",
            actual.mem_checksum, expected.mem_checksum
        ));
    }
    if actual.halted != expected.halted {
        differences.push(format!("halted {} != {}", actual.halted, expected.halted));
    }

    Err(format!("{context}: {}", differences.join(", ")))
}

fn checksum(memory: &[u8]) -> u32 {
    let mut hash = FNV_OFFSET;
    for &byte in &memory[CHECKSUM_START..CHECKSUM_END] {
        hash ^= u32::from(byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

fn snapshot_json(
    ip: u16,
    cs: u16,
    regs: Registers,
    flags: u16,
    mem_checksum: u32,
    halted: bool,
) -> String {
    format!(
        "{{\"ip\":{ip},\"cs\":{cs},\"regs\":{{\"ax\":{},\"bx\":{},\"cx\":{},\"dx\":{},\"sp\":{},\"bp\":{},\"si\":{},\"di\":{},\"ds\":{},\"es\":{},\"ss\":{}}},\"flags\":{flags},\"mem_checksum\":{mem_checksum},\"halted\":{halted}}}",
        regs.ax,
        regs.bx,
        regs.cx,
        regs.dx,
        regs.sp,
        regs.bp,
        regs.si,
        regs.di,
        regs.ds,
        regs.es,
        regs.ss
    )
}

fn parse_snapshot(line: &str) -> Result<Snapshot, String> {
    let mut parser = SnapshotParser { rest: line };
    let mut fields = parser.object(0)?;
    if !parser.rest.trim_matches([' ', '\t', '\r', '\n']).is_empty() {
        return Err("trailing content after snapshot".into());
    }
    let Value::Object(mut regs) = take(&mut fields, "regs")? else {
        return Err("regs must be an object".into());
    };
    let snapshot = Snapshot {
        ip: take_u16(&mut fields, "ip")?,
        cs: take_u16(&mut fields, "cs")?,
        regs: Registers {
            ax: take_u16(&mut regs, "ax")?,
            bx: take_u16(&mut regs, "bx")?,
            cx: take_u16(&mut regs, "cx")?,
            dx: take_u16(&mut regs, "dx")?,
            sp: take_u16(&mut regs, "sp")?,
            bp: take_u16(&mut regs, "bp")?,
            si: take_u16(&mut regs, "si")?,
            di: take_u16(&mut regs, "di")?,
            ds: take_u16(&mut regs, "ds")?,
            es: take_u16(&mut regs, "es")?,
            ss: take_u16(&mut regs, "ss")?,
        },
        flags: take_u16(&mut fields, "flags")?,
        mem_checksum: take_u32(&mut fields, "mem_checksum")?,
        halted: if fields.iter().any(|(key, _)| key == "halted") {
            match take(&mut fields, "halted")? {
                Value::Bool(value) => value,
                _ => return Err("halted must be a boolean".into()),
            }
        } else {
            false
        },
    };
    if !fields.is_empty() || !regs.is_empty() {
        return Err("unknown snapshot or register field".into());
    }
    Ok(snapshot)
}

#[derive(Debug)]
enum Value {
    Number(u32),
    Bool(bool),
    Object(Vec<(String, Value)>),
}

struct SnapshotParser<'a> {
    rest: &'a str,
}

impl SnapshotParser<'_> {
    fn consume(&mut self, token: &str) -> bool {
        self.rest = self.rest.trim_start_matches([' ', '\t', '\r', '\n']);
        if let Some(rest) = self.rest.strip_prefix(token) {
            self.rest = rest;
            true
        } else {
            false
        }
    }

    fn expect(&mut self, token: &str) -> Result<(), String> {
        if self.consume(token) {
            Ok(())
        } else {
            Err(format!("expected `{token}`"))
        }
    }

    fn object(&mut self, depth: u8) -> Result<Vec<(String, Value)>, String> {
        if depth > 1 {
            return Err("snapshot objects may only contain the regs object".into());
        }
        self.expect("{")?;
        let mut fields = Vec::new();
        if self.consume("}") {
            return Ok(fields);
        }
        loop {
            self.expect("\"")?;
            let end = self.rest.find('"').ok_or("unterminated field name")?;
            let key = self.rest[..end].to_string();
            if !key
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte == b'_')
            {
                return Err("field names must use lowercase ASCII and underscores".into());
            }
            self.rest = &self.rest[end + 1..];
            if fields.iter().any(|(name, _)| name == &key) {
                return Err(format!("duplicate field `{key}`"));
            }
            self.expect(":")?;
            let value = if self.consume("true") {
                Value::Bool(true)
            } else if self.consume("false") {
                Value::Bool(false)
            } else if self.rest.starts_with('{') {
                Value::Object(self.object(depth + 1)?)
            } else {
                let end = self.rest.bytes().take_while(u8::is_ascii_digit).count();
                let digits = &self.rest[..end];
                if digits.is_empty() || (digits.len() > 1 && digits.starts_with('0')) {
                    return Err(format!("expected unsigned JSON integer for `{key}`"));
                }
                let number = digits.parse::<u32>().map_err(|error| error.to_string())?;
                self.rest = &self.rest[end..];
                Value::Number(number)
            };
            fields.push((key, value));
            if self.consume("}") {
                return Ok(fields);
            }
            self.expect(",")?;
        }
    }
}

fn take(fields: &mut Vec<(String, Value)>, key: &str) -> Result<Value, String> {
    let index = fields
        .iter()
        .position(|(name, _)| name == key)
        .ok_or_else(|| format!("missing {key}"))?;
    Ok(fields.swap_remove(index).1)
}

fn take_u32(fields: &mut Vec<(String, Value)>, key: &str) -> Result<u32, String> {
    match take(fields, key)? {
        Value::Number(number) => Ok(number),
        _ => Err(format!("{key} must be an unsigned integer")),
    }
}

fn take_u16(fields: &mut Vec<(String, Value)>, key: &str) -> Result<u16, String> {
    u16::try_from(take_u32(fields, key)?).map_err(|error| format!("invalid {key}: {error}"))
}
