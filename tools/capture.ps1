<#
.SYNOPSIS
  Build jk_tdm and run a capture script, without the three ways this goes
  wrong silently.

.DESCRIPTION
  Every one of these has already bitten:

  1. A RUNNING GAME HOLDS THE EXE. The linker fails with "Access is denied
     (os error 5)", cargo exits non-zero - and if the build was piped
     (`cargo build | tail -3`) the shell reports the exit code of the PIPE,
     which is tail's 0. You then capture with a stale binary and read the
     frames as if they proved something. Caught only by noticing the exe
     was older than the source.

  2. FRAMES LAND WHERE NOBODY LOOKS. The snap path used to be relative to
     the working directory, so a run launched from engine/ wrote into
     engine/handback/ and exited 0. Fixed in-engine now (capture_dir
     anchors to CARGO_MANIFEST_DIR), but this script still asserts the
     frames actually appeared where they were expected.

  3. STALE FRAMES READ AS FRESH. If a capture writes nothing, the previous
     run's PNGs are still sitting there and look like a result.

  So: kill first, check the REAL exit code, assert the binary is newer
  than every source file, clear the target directory, and verify the
  frames are both present and newly written.

.EXAMPLE
  pwsh tools/capture.ps1 menus
  pwsh tools/capture.ps1 baseline -SkipBuild
#>
[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)][string]$Script,
    [switch]$SkipBuild,
    [int]$TimeoutSec = 120
)

$ErrorActionPreference = 'Stop'
$repo   = Split-Path -Parent $PSScriptRoot
$engine = Join-Path $repo 'engine'
$crate  = Join-Path $engine 'crates\jk_tdm'
$exe    = Join-Path $engine 'target\release\jk_tdm.exe'
$outDir = Join-Path $crate "handback\brief-vii\$Script"

function Fail($msg) { Write-Host "FAIL: $msg" -ForegroundColor Red; exit 1 }
function Step($msg) { Write-Host "==> $msg" -ForegroundColor Cyan }

# --- 1. nothing may hold the exe -------------------------------------------
$held = Get-Process jk_tdm -ErrorAction SilentlyContinue
if ($held) {
    Step "stopping $($held.Count) running instance(s): $($held.Id -join ', ')"
    $held | ForEach-Object { $_.Kill(); $_.WaitForExit(10000) | Out-Null }
    Start-Sleep -Milliseconds 300
}

# --- 2. build, and read the REAL exit code ---------------------------------
if (-not $SkipBuild) {
    Step 'cargo build --release -p jk_tdm'
    # No pipe. The exit code must come from cargo, not from whatever it
    # would have been piped into.
    & cargo build --release -p jk_tdm --manifest-path (Join-Path $engine 'Cargo.toml')
    if ($LASTEXITCODE -ne 0) { Fail "cargo exited $LASTEXITCODE" }
}

if (-not (Test-Path $exe)) { Fail "no binary at $exe" }

# --- 3. the binary must be newer than every source -------------------------
$exeTime = (Get-Item $exe).LastWriteTime
$newer = Get-ChildItem (Join-Path $crate 'src') -Recurse -File -Filter *.rs |
         Where-Object { $_.LastWriteTime -gt $exeTime }
if ($newer) {
    Fail ("binary is STALE - built {0:HH:mm:ss} but these are newer:`n  {1}" -f
          $exeTime, (($newer | ForEach-Object { "$($_.Name) $($_.LastWriteTime.ToString('HH:mm:ss'))" }) -join "`n  "))
}
Step ("binary is fresh ({0:HH:mm:ss})" -f $exeTime)

# --- 4. clear the target dir so stale frames cannot pass as fresh ----------
if (Test-Path $outDir) { Get-ChildItem $outDir -File | Remove-Item -Force }
$before = Get-Date

# --- 5. run --------------------------------------------------------------
Step "JK_CAPTURE=$Script"
$env:JK_CAPTURE = $Script
try {
    $p = Start-Process -FilePath $exe -WorkingDirectory $engine -PassThru
    if (-not $p.WaitForExit($TimeoutSec * 1000)) {
        $p.Kill(); Fail "capture timed out after ${TimeoutSec}s"
    }
    if ($p.ExitCode -ne 0) { Fail "game exited $($p.ExitCode)" }
} finally {
    Remove-Item Env:\JK_CAPTURE -ErrorAction SilentlyContinue
}

# --- 6. frames must exist AND be from this run -----------------------------
if (-not (Test-Path $outDir)) { Fail "no output directory: $outDir" }
$frames = Get-ChildItem $outDir -Filter *.png | Where-Object { $_.LastWriteTime -ge $before }
if (-not $frames) { Fail "the run exited 0 but wrote NO frames to $outDir" }

# a run that writes to the wrong place is the bug this script exists for
$stray = Join-Path $engine 'handback'
if (Test-Path $stray) { Fail "frames leaked to $stray - capture_dir regressed" }

Step "$($frames.Count) fresh frame(s):"
$frames | Sort-Object Name | ForEach-Object {
    "    {0,-30} {1,6} KB  {2:HH:mm:ss}" -f $_.Name, [math]::Round($_.Length / 1KB), $_.LastWriteTime
}
Write-Host "OK" -ForegroundColor Green
