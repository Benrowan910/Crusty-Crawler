# Uninstallation script for Crusty-Crawler (Windows)

#Requires -RunAsAdministrator

$ErrorActionPreference = "Stop"

$APP_NAME = "Crusty-Crawler"
$INSTALL_DIR = "$env:ProgramFiles\$APP_NAME"
$DESKTOP = [Environment]::GetFolderPath("CommonDesktopDirectory")
$START_MENU = "$env:ProgramData\Microsoft\Windows\Start Menu\Programs"

Write-Host "🦀 Crusty-Crawler Uninstallation Script" -ForegroundColor Cyan
Write-Host ""

# Check if running as administrator
$isAdmin = ([Security.Principal.WindowsPrincipal] [Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
if (-not $isAdmin) {
    Write-Host "❌ Please run this script as Administrator" -ForegroundColor Red
    exit 1
}

Write-Host "🗑️  Uninstalling Crusty-Crawler..." -ForegroundColor Yellow

# Stop any running instances
Write-Host "⏹️  Stopping any running instances..." -ForegroundColor Yellow
Get-Process -Name "crusty-crawler" -ErrorAction SilentlyContinue | Stop-Process -Force

# Remove from PATH
Write-Host "🔗 Removing from system PATH..." -ForegroundColor Yellow
$currentPath = [Environment]::GetEnvironmentVariable("Path", "Machine")
$newPath = ($currentPath.Split(';') | Where-Object { $_ -ne $INSTALL_DIR }) -join ';'
[Environment]::SetEnvironmentVariable("Path", $newPath, "Machine")

# Remove desktop shortcut
if (Test-Path "$DESKTOP\$APP_NAME.lnk") {
    Write-Host "🖥️  Removing desktop shortcut..." -ForegroundColor Yellow
    Remove-Item "$DESKTOP\$APP_NAME.lnk" -Force
}

# Remove Start Menu shortcut
if (Test-Path "$START_MENU\$APP_NAME.lnk") {
    Write-Host "📋 Removing Start Menu shortcut..." -ForegroundColor Yellow
    Remove-Item "$START_MENU\$APP_NAME.lnk" -Force
}

# Remove Windows Firewall rule
Write-Host "🔥 Removing Windows Firewall rule..." -ForegroundColor Yellow
Remove-NetFirewallRule -DisplayName "Crusty-Crawler" -ErrorAction SilentlyContinue

# Remove installation directory
if (Test-Path $INSTALL_DIR) {
    Write-Host "📁 Removing installation directory..." -ForegroundColor Yellow
    Remove-Item -Recurse -Force $INSTALL_DIR
}

Write-Host ""
Write-Host "✅ Uninstallation completed successfully!" -ForegroundColor Green
Write-Host "👋 Crusty-Crawler has been removed from your system." -ForegroundColor Cyan
Write-Host ""
pause
