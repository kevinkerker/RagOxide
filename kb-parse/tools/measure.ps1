# Measurement of the parser on the corpus (A60): per file the duration of a first and a
# warm run, seconds per page and the peak working set of the worker process, then the
# time from a cancel to the return of the call. Prints a Markdown table.
#
# Prerequisites: the release build (`cargo build -p kb-parse --release`) and
# RAGOXIDE_TEST_MODELS_DIR pointing at the copy of the docling.rs model release.
# Run from the repository root:
#
#   powershell -NoProfile -ExecutionPolicy Bypass -File kb-parse\tools\measure.ps1
#
# Each call starts its own worker; the script polls the worker's PeakWorkingSet64 every
# 50 ms, which Windows keeps as the maximum, so the last sample is the peak up to then.

$ErrorActionPreference = 'Stop'
$dump = Join-Path (Get-Location) 'target\release\kb-parse-dump.exe'
if (-not (Test-Path $dump)) { throw "missing $dump, run cargo build -p kb-parse --release" }
if (-not $env:RAGOXIDE_TEST_MODELS_DIR) { throw 'RAGOXIDE_TEST_MODELS_DIR is not set' }
$out = Join-Path ([IO.Path]::GetTempPath()) ('kb-parse-measure-' + $PID)
New-Item -ItemType Directory -Force $out | Out-Null

function Invoke-Dump([string]$file, [string[]]$extra) {
    $json = Join-Path $out 'out.json'
    $err = Join-Path $out 'err.txt'
    $arguments = @('"' + $file + '"') + $extra
    $p = Start-Process -FilePath $dump -ArgumentList $arguments -NoNewWindow -PassThru `
        -RedirectStandardOutput $json -RedirectStandardError $err
    $peak = 0
    while (-not $p.HasExited) {
        foreach ($w in @(Get-Process -Name 'kb-parse-worker' -ErrorAction SilentlyContinue)) {
            try { if ($w.PeakWorkingSet64 -gt $peak) { $peak = $w.PeakWorkingSet64 } } catch { }
        }
        Start-Sleep -Milliseconds 50
    }
    $p.WaitForExit()
    $text = Get-Content $err -Raw
    $duration = [double]::NaN
    if ($text -match 'duration: ([0-9.]+) s') { $duration = [double]$Matches[1] }
    $cancel = [double]::NaN
    if ($text -match 'cancel to return: ([0-9.]+) s') { $cancel = [double]$Matches[1] }
    $pages = 0
    if ((Get-Item $json).Length -gt 0) {
        $doc = Get-Content $json -Raw -Encoding UTF8 | ConvertFrom-Json
        if ($doc.pages) { $pages = @($doc.pages).Count }
    }
    [pscustomobject]@{ Duration = $duration; Peak = $peak; Pages = $pages; Cancel = $cancel }
}

$inv = [Globalization.CultureInfo]::InvariantCulture
$cpu = (Get-CimInstance Win32_Processor | Select-Object -First 1)
'Prozessor: {0}, {1} Kerne, {2} logische Prozessoren; docling.rs 2 Worker x 2 Threads; {3}' -f `
    $cpu.Name.Trim(), $cpu.NumberOfCores, $cpu.NumberOfLogicalProcessors, (Get-Date -Format 'yyyy-MM-dd')
''
'| Datei | Seiten | erster Lauf s | warmer Lauf s | s/Seite warm | Spitze Worker MB |'
'| --- | --- | --- | --- | --- | --- |'
foreach ($f in Get-ChildItem 'corpus\documents' | Sort-Object Name) {
    $first = Invoke-Dump $f.FullName @()
    $warm = Invoke-Dump $f.FullName @()
    $perPage = '-'
    if ($warm.Pages -gt 0) { $perPage = ($warm.Duration / $warm.Pages).ToString('0.00', $inv) }
    $peak = [math]::Max($first.Peak, $warm.Peak) / 1MB
    '| {0} | {1} | {2} | {3} | {4} | {5} |' -f $f.Name, $warm.Pages, `
        $first.Duration.ToString('0.00', $inv), $warm.Duration.ToString('0.00', $inv), $perPage, `
        $peak.ToString('0', $inv)
}
''
$amtsblatt = Get-Item 'corpus\documents\de-eu-amtsblatt-l-141-2013.pdf'
foreach ($after in 3000, 10000) {
    $c = Invoke-Dump $amtsblatt.FullName @('--cancel-after', "$after")
    # The script stays ASCII (PowerShell 5.1 reads it as ANSI); the umlaut is built.
    'Abbruch nach {0} ms im Amtsblatt (60 Seiten): R{2}ckkehr nach {1} s' -f $after, $c.Cancel.ToString('0.000', $inv), [char]0x00FC
}
Remove-Item -Recurse -Force $out
