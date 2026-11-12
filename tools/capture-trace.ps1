[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [ValidatePattern('^[A-Za-z0-9](?:[A-Za-z0-9._-]*[A-Za-z0-9_-])?$')]
    [string]$Name,

    [Parameter(Mandatory = $true)]
    [string]$Program,

    [Parameter(Mandatory = $true)]
    [string]$Trace,

    [Parameter(Mandatory = $true)]
    [string]$Final,

    [string]$Source,

    [string]$GoldenRoot = (Join-Path $PSScriptRoot '..\tests\golden'),

    [switch]$SkipChecksumCheck
)

$ErrorActionPreference = 'Stop'

$registerNames = @('ax', 'bx', 'cx', 'dx', 'sp', 'bp', 'si', 'di', 'ds', 'es', 'ss')

function Read-ObjectFields {
    param([System.Text.Json.JsonElement]$Element, [string[]]$Required, [string[]]$Optional = @())

    if ($Element.ValueKind -ne [System.Text.Json.JsonValueKind]::Object) {
        throw 'snapshot and regs must be JSON objects'
    }
    $fields = [Collections.Generic.Dictionary[string, System.Text.Json.JsonElement]]::new([StringComparer]::Ordinal)
    foreach ($property in $Element.EnumerateObject()) {
        if ($fields.ContainsKey($property.Name)) {
            throw "duplicate field: $($property.Name)"
        }
        if ($property.Name -cnotin $Required -and $property.Name -cnotin $Optional) {
            throw "unknown field: $($property.Name)"
        }
        $fields.Add($property.Name, $property.Value)
    }
    foreach ($key in $Required) {
        if (-not $fields.ContainsKey($key)) {
            throw "missing field: $key"
        }
    }
    return ,$fields
}

function Read-UnsignedInteger {
    param([System.Text.Json.JsonElement]$Element, [uint32]$Maximum, [string]$Label)

    [uint32]$number = 0
    if ($Element.ValueKind -ne [System.Text.Json.JsonValueKind]::Number -or -not $Element.TryGetUInt32([ref]$number) -or $number -gt $Maximum) {
        throw "$Label must be an unsigned integer from 0 to $Maximum"
    }
    return $number
}

function Read-Snapshots {
    param([string]$Path, [string]$Label)

    $lines = @(Get-Content -LiteralPath $Path | Where-Object { $_.Trim().Length -gt 0 })
    if ($lines.Count -eq 0) {
        throw "$Label is empty: $Path"
    }
    $index = 0
    foreach ($line in $lines) {
        $index++
        $document = $null
        try {
            $document = [System.Text.Json.JsonDocument]::Parse([string]$line)
            $fields = Read-ObjectFields $document.RootElement @('ip', 'cs', 'regs', 'flags', 'mem_checksum') @('halted')
            $regs = Read-ObjectFields $fields['regs'] $registerNames
            $snapshot = [ordered]@{}
            foreach ($key in @('ip', 'cs')) {
                $snapshot[$key] = Read-UnsignedInteger $fields[$key] 65535 $key
            }
            $snapshot['regs'] = [ordered]@{}
            foreach ($key in $registerNames) {
                $snapshot['regs'][$key] = Read-UnsignedInteger $regs[$key] 65535 "regs.$key"
            }
            $snapshot['flags'] = Read-UnsignedInteger $fields['flags'] 65535 'flags'
            $snapshot['mem_checksum'] = Read-UnsignedInteger $fields['mem_checksum'] 4294967295 'mem_checksum'
            $snapshot['halted'] = $false
            if ($fields.ContainsKey('halted')) {
                if ($fields['halted'].ValueKind -notin @([System.Text.Json.JsonValueKind]::True, [System.Text.Json.JsonValueKind]::False)) {
                    throw 'halted must be true or false'
                }
                $snapshot['halted'] = $fields['halted'].GetBoolean()
            }
            if ($Label -eq 'trace' -and $snapshot['halted']) {
                throw 'trace rows must describe running states before instructions'
            }
            $snapshot
        }
        catch {
            throw "$Label line ${index}: $_"
        }
        finally {
            if ($null -ne $document) {
                $document.Dispose()
            }
        }
    }
}

function Get-InitialChecksum {
    param([byte[]]$ProgramBytes)

    if ($ProgramBytes.Length -gt 0xFF00) {
        throw "program is $($ProgramBytes.Length) bytes, but the M1 checksum window holds at most 65280"
    }

    $memory = New-Object byte[] 65536
    [Array]::Copy($ProgramBytes, 0, $memory, 0x0100, $ProgramBytes.Length)

    [uint32]$hash = 2166136261
    for ($offset = 0x0100; $offset -lt 0x10000; $offset++) {
        $hash = $hash -bxor $memory[$offset]
        $hash = [uint32]((([uint64]$hash * 0x01000193) -band 4294967295))
    }

    return $hash
}

if (-not (Test-Path -LiteralPath $Program -PathType Leaf)) {
    throw "program not found: $Program"
}
if (-not (Test-Path -LiteralPath $Trace -PathType Leaf)) {
    throw "trace not found: $Trace"
}
if (-not (Test-Path -LiteralPath $Final -PathType Leaf)) {
    throw "final state not found: $Final"
}
if ($Source -and -not (Test-Path -LiteralPath $Source -PathType Leaf)) {
    throw "source not found: $Source"
}

$programExtension = [IO.Path]::GetExtension($Program).ToLowerInvariant()
if ($programExtension -notin @('.com', '.bin')) {
    throw "program must be a .com or .bin file, got $programExtension"
}

$traceSnapshots = @(Read-Snapshots -Path $Trace -Label 'trace')
$finalSnapshots = @(Read-Snapshots -Path $Final -Label 'final state')
if ($finalSnapshots.Count -ne 1) {
    throw 'final state must contain exactly one JSON object'
}
$programBytes = [IO.File]::ReadAllBytes((Resolve-Path -LiteralPath $Program))
if ($programBytes.Length -gt 0xFF00) {
    throw "program is $($programBytes.Length) bytes, but the M1 checksum window holds at most 65280"
}
if (-not $SkipChecksumCheck) {
    $expected = Get-InitialChecksum -ProgramBytes $programBytes
    $actual = [uint32]$traceSnapshots[0]['mem_checksum']
    if ($expected -ne $actual) {
        throw ('Initial mem_checksum is 0x{0:X8}, expected 0x{1:X8}. The M1 harness requires zero-filled memory and the program at physical 0x0100.' -f $actual, $expected)
    }
}

$destination = [IO.Path]::GetFullPath((Join-Path $GoldenRoot $Name))
if (Test-Path -LiteralPath $destination) {
    throw "capture destination already exists: $destination"
}
New-Item -ItemType Directory -Path $destination | Out-Null

$programName = if ($programExtension -eq '.bin') { 'program.bin' } else { 'program.com' }
[IO.File]::WriteAllBytes((Join-Path $destination $programName), $programBytes)
$traceSnapshots | ForEach-Object { ConvertTo-Json -InputObject $_ -Depth 3 -Compress } | Set-Content -LiteralPath (Join-Path $destination 'trace.jsonl') -Encoding utf8NoBOM
ConvertTo-Json -InputObject $finalSnapshots[0] -Depth 3 -Compress | Set-Content -LiteralPath (Join-Path $destination 'final.json') -Encoding utf8NoBOM
if ($Source) {
    Copy-Item -LiteralPath $Source -Destination (Join-Path $destination 'program.asm')
}

Write-Host "Installed golden trace '$Name' in $destination"
Write-Host 'Run: cargo test -p emu86-core --test golden'
