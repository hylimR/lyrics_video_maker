
# Usage: ./run_cli.ps1 <input_ass_path> <output_video_path> [duration] [-Verbose]

[CmdletBinding()]
param (
    [Parameter(Mandatory = $true)]
    [string]$InputPath,

    [Parameter(Mandatory = $true)]
    [string]$OutputPath,

    [string]$Duration
)

$ErrorActionPreference = "Stop"

# Ensure scripts exist
if (-not (Test-Path "scripts/ass_to_json.js")) {
    Write-Error "scripts/ass_to_json.js not found."
    exit 1
}

$TempJson = "temp_render.json"

Write-Verbose "Converting ASS to JSON..."
node scripts/ass_to_json.js $InputPath $TempJson

if (-not (Test-Path $TempJson)) {
    Write-Error "JSON conversion failed."
    exit 1
}

if ($PSBoundParameters['Verbose']) {
    $DebugFile = "$OutputPath.debug.json"
    Copy-Item $TempJson $DebugFile -Force
    Write-Host "Debug JSON saved to $DebugFile"
}

Write-Verbose "Running Render CLI..."

$Args = @("--manifest-path", "src-tauri/Cargo.toml", "--bin", "render_cli", "--", $TempJson, $OutputPath)

if ($Duration) {
    $Args += $Duration
}

& cargo run $Args

# Clean up
if (Test-Path $TempJson) {
    Remove-Item $TempJson
}

Write-Verbose "Done."
