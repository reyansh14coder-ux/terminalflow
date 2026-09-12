# TerminalFlow Installer for Windows

Write-Host "🚀 Installing TerminalFlow..." -ForegroundColor Cyan

# Detect architecture
$arch = if ([Environment]::Is64BitOperatingSystem) { "amd64" } else { "x86" }

# Download URL
$downloadUrl = "https://github.com/YOUR_USERNAME/terminalflow/releases/latest/download/terminalflow-windows-$arch.exe"

# Install directory
$installDir = "$env:LOCALAPPDATA\TerminalFlow"
$exePath = "$installDir\terminalflow.exe"

# Create install directory
if (!(Test-Path $installDir)) {
    New-Item -ItemType Directory -Force -Path $installDir | Out-Null
}

Write-Host "📥 Downloading TerminalFlow..." -ForegroundColor Yellow
Invoke-WebRequest -Uri $downloadUrl -OutFile $exePath

# Add to PATH
$currentPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($currentPath -notlike "*$installDir*") {
    [Environment]::SetEnvironmentVariable("Path", "$currentPath;$installDir", "User")
    Write-Host "✅ Added to PATH" -ForegroundColor Green
}

Write-Host ""
Write-Host "✅ TerminalFlow installed successfully!" -ForegroundColor Green
Write-Host ""
Write-Host "🚀 Run 'terminalflow' or 'tf' to start!" -ForegroundColor Cyan
Write-Host "📚 Documentation: https://github.com/YOUR_USERNAME/terminalflow#readme" -ForegroundColor Gray
