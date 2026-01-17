
Write-Host "🚀 Building KLyric Studio (Release)..." -ForegroundColor Cyan

# 1. Build release binary
cargo build --release -p klyric-gui

if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ Build failed!" -ForegroundColor Red
    exit $LASTEXITCODE
}

# 2. Create dist folder
$distDir = "dist"
if (Test-Path $distDir) {
    Remove-Item $distDir -Recurse -Force
}
New-Item -ItemType Directory -Path $distDir | Out-Null

# 3. Copy binary
$exeSource = "target/release/klyric-gui.exe"
if (Test-Path $exeSource) {
    Copy-Item $exeSource -Destination $distDir
    Write-Host "✅ Copied klyric-gui.exe" -ForegroundColor Green
}
else {
    Write-Host "❌ executable not found at $exeSource" -ForegroundColor Red
    exit 1
}

# 4. Copy samples folder
$samplesSource = "samples"
if (Test-Path $samplesSource) {
    Copy-Item $samplesSource -Destination "$distDir/samples" -Recurse
    Write-Host "✅ Copied samples/ assets" -ForegroundColor Green
}
else {
    Write-Host "⚠️ Warning: samples/ folder not found at $samplesSource" -ForegroundColor Yellow
}

Write-Host "✨ Build complete! Distribution ready in '$distDir/'" -ForegroundColor Cyan
