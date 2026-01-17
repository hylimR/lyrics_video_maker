# KLyric Studio Startup Script

Write-Host "🚀 Starting KLyric Studio..." -ForegroundColor Cyan

# Check if klyric-gui is available
if (Test-Path "crates/klyric-gui") {
    Write-Host "📦 Found klyric-gui crate. Building and running..." -ForegroundColor Gray
    cargo run -p klyric-gui
} else {
    Write-Host "❌ Error: crates/klyric-gui not found. Please ensure you are in the project root." -ForegroundColor Red
    exit 1
}
