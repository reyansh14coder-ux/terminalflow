# TerminalFlow Installer for Windows

$ErrorActionPreference = "Stop"

Write-Host "🚀 Installing TerminalFlow..." -ForegroundColor Cyan

# Detect architecture
$arch = if ([Environment]::Is64BitOperatingSystem) { "amd64" } else { "x86" }

# Download URL
$downloadUrl = "https://github.com/reyansh14coder-ux/terminalflow/releases/latest/download/terminalflow-windows-$arch.exe"

# Install directory
$installDir = "$env:LOCALAPPDATA\TerminalFlow"
$exePath = "$installDir\terminalflow.exe"

# Create install directory
if (!(Test-Path $installDir)) {
    New-Item -ItemType Directory -Force -Path $installDir | Out-Null
}

Write-Host "📥 Downloading TerminalFlow from: $downloadUrl" -ForegroundColor Yellow

try {
    [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
    $ProgressPreference = 'SilentlyContinue'
    Invoke-WebRequest -Uri $downloadUrl -OutFile $exePath -UseBasicParsing -TimeoutSec 120
} catch {
    Write-Host "❌ Download failed: $_" -ForegroundColor Red
    Write-Host ""
    Write-Host "This may be because no release binaries are available yet." -ForegroundColor Yellow
    Write-Host "You can build from source instead:" -ForegroundColor Yellow
    Write-Host "  git clone https://github.com/reyansh14coder-ux/terminalflow.git" -ForegroundColor White
    Write-Host "  cd terminalflow" -ForegroundColor White
    Write-Host "  cargo build --release" -ForegroundColor White
    Write-Host "  Copy-Item target\release\terminalflow.exe `$env:LOCALAPPDATA\TerminalFlow\" -ForegroundColor White
    exit 1
}

# Verify download
if (!(Test-Path $exePath) -or (Get-Item $exePath).Length -lt 1MB) {
    Write-Host "❌ Downloaded file is missing or too small" -ForegroundColor Red
    exit 1
}

# Add to PATH
$currentPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($currentPath -notlike "*$installDir*") {
    [Environment]::SetEnvironmentVariable("Path", "$currentPath;$installDir", "User")
    $env:Path = "$env:Path;$installDir"
    Write-Host "✅ Added to PATH" -ForegroundColor Green
}

Write-Host ""
Write-Host "✅ TerminalFlow installed successfully!" -ForegroundColor Green
Write-Host ""
Write-Host "🚀 Run 'terminalflow' or 'tf' to start!" -ForegroundColor Cyan
Write-Host "📚 Documentation: https://github.com/reyansh14coder-ux/terminalflow#readme" -ForegroundColor Gray
