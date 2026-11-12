$ErrorActionPreference = 'Stop'
$importer = Join-Path $PSScriptRoot '..\capture-trace.ps1'
$tempBase = [IO.Path]::GetFullPath([IO.Path]::GetTempPath())
$testRoot = Join-Path $tempBase ('emu86-capture-tests-' + [guid]::NewGuid().ToString('N'))
$null = New-Item -ItemType Directory -Path $testRoot
$failures = [Collections.Generic.List[string]]::new()
$caseIndex = 0

function Assert-Rejected {
    param([string]$Label, [string]$Snapshot, [string]$FinalState = $valid)

    $script:caseIndex++
    Set-Content -LiteralPath $tracePath -Value $Snapshot -Encoding utf8
    Set-Content -LiteralPath $finalPath -Value $FinalState -Encoding utf8
    $name = 'invalid-' + $script:caseIndex
    try {
        & $importer -Name $name -Program $programPath -Trace $tracePath -Final $finalPath -GoldenRoot $outputRoot -SkipChecksumCheck
        $failures.Add("accepted $Label")
    }
    catch {
        if (Test-Path -LiteralPath (Join-Path $outputRoot $name)) {
            $failures.Add("created destination for $Label")
        }
    }
}

function Assert-RejectedMemory {
    param([string]$Label, [byte[]]$MemoryBytes, [string]$Snapshot = $memoryInitial)

    $script:caseIndex++
    [IO.File]::WriteAllBytes($memoryPath, $MemoryBytes)
    Set-Content -LiteralPath $tracePath -Value $Snapshot -Encoding utf8
    Set-Content -LiteralPath $finalPath -Value $memoryFinal -Encoding utf8
    $name = 'invalid-memory-' + $script:caseIndex
    try {
        & $importer -Name $name -Program $programPath -Trace $tracePath -Final $finalPath -InitialMemory $memoryPath -GoldenRoot $outputRoot -SkipChecksumCheck
        $failures.Add("accepted $Label")
    }
    catch {
        if (Test-Path -LiteralPath (Join-Path $outputRoot $name)) {
            $failures.Add("created destination for $Label")
        }
    }
}

try {
    $programPath = Join-Path $testRoot 'input.com'
    $tracePath = Join-Path $testRoot 'trace.jsonl'
    $finalPath = Join-Path $testRoot 'final.json'
    $outputRoot = Join-Path $testRoot 'golden'
    [IO.File]::WriteAllBytes($programPath, [byte[]]@(0x90, 0xF4))
    $valid = '{"ip":256,"cs":0,"regs":{"ax":0,"bx":0,"cx":0,"dx":0,"sp":0,"bp":0,"si":0,"di":0,"ds":0,"es":0,"ss":0},"flags":61442,"mem_checksum":0,"halted":false}'

    Assert-Rejected 'fractional register' ($valid.Replace('"ax":0', '"ax":0.5'))
    Assert-Rejected 'string register' ($valid.Replace('"ax":0', '"ax":"0"'))
    Assert-Rejected 'null register' ($valid.Replace('"ax":0', '"ax":null'))
    Assert-Rejected 'boolean register' ($valid.Replace('"ax":0', '"ax":false'))
    Assert-Rejected 'negative register' ($valid.Replace('"ax":0', '"ax":-1'))
    Assert-Rejected 'out-of-range register' ($valid.Replace('"ax":0', '"ax":65536'))
    Assert-Rejected 'missing register' ($valid.Replace('"ax":0,', ''))
    Assert-Rejected 'unknown field' ($valid.Replace('"cs":0', '"cs":0,"extra":1'))
    Assert-Rejected 'unknown register' ($valid.Replace('"ax":0', '"ax":0,"extra":1'))
    Assert-Rejected 'case-mismatched field' ($valid.Replace('"cs":0', '"CS":0'))
    Assert-Rejected 'duplicate field' ($valid.Replace('"cs":0', '"cs":0,"cs":1'))
    Assert-Rejected 'array snapshot' ('[' + $valid + ']')
    Assert-Rejected 'halted pre-instruction row' ($valid.Replace('false', 'true'))
    Assert-Rejected 'multiple final states' $valid ($valid + "`n" + $valid)

    Set-Content -LiteralPath $tracePath -Value $valid -Encoding utf8
    Set-Content -LiteralPath $finalPath -Value $valid -Encoding utf8
    foreach ($name in @('.', '..')) {
        try {
            & $importer -Name $name -Program $programPath -Trace $tracePath -Final $finalPath -GoldenRoot $outputRoot -SkipChecksumCheck
            $failures.Add("accepted unsafe name $name")
        }
        catch {}
    }

    [IO.File]::WriteAllBytes($programPath, [byte[]]::new(0xFF01))
    Assert-Rejected 'oversized binary with checksum bypass' $valid
    [IO.File]::WriteAllBytes($programPath, [byte[]]@(0x90, 0xF4))

    $valid = $valid.Replace('"mem_checksum":0', '"mem_checksum":341274553')
    Set-Content -LiteralPath $tracePath -Value $valid.Replace(':', ' : ') -Encoding utf8
    Set-Content -LiteralPath $finalPath -Value $valid.Replace('"ip":256', '"ip":257') -Encoding utf8
    & $importer -Name 'valid' -Program $programPath -Trace $tracePath -Final $finalPath -GoldenRoot $outputRoot
    $installed = Join-Path $outputRoot 'valid'
    if (-not (Test-Path -LiteralPath (Join-Path $installed 'program.com'))) {
        $failures.Add('valid capture was not installed')
    }
    try {
        & $importer -Name 'valid' -Program $programPath -Trace $tracePath -Final $finalPath -GoldenRoot $outputRoot
        $failures.Add('overwrote an existing capture')
    }
    catch {}

    $memoryPath = Join-Path $testRoot 'memory.bin'
    $memory = [byte[]]::new(65536)
    $memory[0x0200] = 0x34
    $memory[0x0201] = 0x12
    $memoryProgram = [byte[]]@(0xA1, 0x00, 0x02, 0x90)
    [Array]::Copy($memoryProgram, 0, $memory, 0x1100, $memoryProgram.Length)
    [IO.File]::WriteAllBytes($programPath, $memoryProgram)
    $memoryInitial = $valid.Replace('"cs":0', '"cs":256').Replace('"flags":61442', '"flags":514').Replace('341274553', '3977269960')
    $memoryFinal = $memoryInitial.Replace('"ip":256', '"ip":259').Replace('"ax":0', '"ax":4660')
    foreach ($size in @(0, 65535, 65537)) {
        Assert-RejectedMemory "memory dump size $size" ([byte[]]::new($size))
    }
    Assert-RejectedMemory 'mismatched program in memory' ([byte[]]::new(65536))
    Assert-RejectedMemory 'entry below checksum window' $memory ($memoryInitial.Replace('"cs":256', '"cs":0').Replace('"ip":256', '"ip":255'))
    Assert-RejectedMemory 'entry above checksum window' $memory ($memoryInitial.Replace('"cs":256', '"cs":4096').Replace('"ip":256', '"ip":0'))
    Assert-RejectedMemory 'program crosses checksum window end' $memory ($memoryInitial.Replace('"cs":256', '"cs":0').Replace('"ip":256', '"ip":65534'))
    [IO.File]::WriteAllBytes($programPath, [byte[]]::new(0))
    Assert-RejectedMemory 'empty program with memory dump' $memory
    [IO.File]::WriteAllBytes($programPath, $memoryProgram)
    [IO.File]::WriteAllBytes($memoryPath, $memory)
    Set-Content -LiteralPath $tracePath -Value $memoryInitial -Encoding utf8
    Set-Content -LiteralPath $finalPath -Value $memoryFinal -Encoding utf8
    & $importer -Name 'valid-memory' -Program $programPath -Trace $tracePath -Final $finalPath -InitialMemory $memoryPath -GoldenRoot $outputRoot
    $installedMemory = [IO.File]::ReadAllBytes((Join-Path $outputRoot 'valid-memory/initial-memory.bin'))
    if ([Convert]::ToHexString($installedMemory) -cne [Convert]::ToHexString($memory)) {
        $failures.Add('initial memory was not preserved')
    }

    $previousRoot = $env:EMU86_GOLDEN_ROOT
    try {
        $env:EMU86_GOLDEN_ROOT = $outputRoot
        Push-Location (Join-Path $PSScriptRoot '..\..')
        try {
            cargo test -p emu86-core --test golden replays_committed_golden_traces -- --nocapture
            if ($LASTEXITCODE -ne 0) {
                $failures.Add('Rust harness rejected the imported synthetic capture')
            }
        }
        finally {
            Pop-Location
        }
    }
    finally {
        $env:EMU86_GOLDEN_ROOT = $previousRoot
    }

    if ($failures.Count -gt 0) {
        throw ($failures -join "`n")
    }
    Write-Host 'Capture importer tests passed.'
}
finally {
    $resolved = [IO.Path]::GetFullPath($testRoot)
    if ($resolved.StartsWith($tempBase, [StringComparison]::OrdinalIgnoreCase) -and (Split-Path $resolved -Leaf).StartsWith('emu86-capture-tests-')) {
        Remove-Item -LiteralPath $resolved -Recurse -Force
    }
}
