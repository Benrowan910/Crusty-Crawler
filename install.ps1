# Installation script for Crusty-Crawler (Windows)
# Installs the application to Program Files and creates Windows Service (optional)

#Requires -RunAsAdministrator

$ErrorActionPreference = "Stop"

$APP_NAME = "Crusty-Crawler"
$INSTALL_DIR = "$env:ProgramFiles\$APP_NAME"
$DESKTOP = [Environment]::GetFolderPath("CommonDesktopDirectory")

Write-Host "Crusty-Crawler Installation Script" -ForegroundColor Cyan
Write-Host ""

# Check if running as administrator
$isAdmin = ([Security.Principal.WindowsPrincipal] [Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
if (-not $isAdmin) {
    Write-Host "[ERROR] Please run this script as Administrator" -ForegroundColor Red
    Write-Host "Right-click PowerShell and select Run as Administrator" -ForegroundColor Yellow
    exit 1
}

# Ask user for installation mode
Write-Host "Installation Mode Selection" -ForegroundColor Yellow
Write-Host "1. GUI Mode (Desktop application with graphical interface)"
Write-Host "2. CLI Mode (Command-line interface for servers)"
Write-Host ""
$mode = Read-Host "Select installation mode (1 or 2)"

$MODE = "gui"
switch ($mode) {
    "1" {
        $MODE = "gui"
        Write-Host "Installing GUI mode..." -ForegroundColor Green
    }
    "2" {
        $MODE = "cli"
        Write-Host "Installing CLI mode..." -ForegroundColor Green
    }
    default {
        Write-Host "Invalid selection. Defaulting to GUI mode." -ForegroundColor Yellow
        $MODE = "gui"
    }
}

Write-Host ""
Write-Host "[INSTALL] Installing Crusty-Crawler..." -ForegroundColor Yellow

# Create installation directory
Write-Host "[INSTALL] Creating installation directory..." -ForegroundColor Yellow
if (Test-Path $INSTALL_DIR) {
    Remove-Item -Recurse -Force $INSTALL_DIR
}
New-Item -ItemType Directory -Path $INSTALL_DIR | Out-Null

# Copy binary
Write-Host "[INSTALL] Installing application..." -ForegroundColor Yellow
$exeName = "crusty-crawler.exe"
if (Test-Path $exeName) {
    Copy-Item $exeName "$INSTALL_DIR\"
} else {
    Write-Host "[ERROR] Binary not found: $exeName" -ForegroundColor Red
    Write-Host "Make sure you are in the correct directory." -ForegroundColor Yellow
    exit 1
}

# Copy assets and public directory
Write-Host "[INSTALL] Installing assets..." -ForegroundColor Yellow
if (Test-Path "Assets") {
    Copy-Item -Recurse "Assets" "$INSTALL_DIR\"
}
if (Test-Path "public") {
    Copy-Item -Recurse "public" "$INSTALL_DIR\"
}

# Copy documentation
if (Test-Path "LICENSE") {
    Copy-Item "LICENSE" "$INSTALL_DIR\"
}
if (Test-Path "README.md") {
    Copy-Item "README.md" "$INSTALL_DIR\"
}

# Copy wrapper script if available
if (Test-Path "crwlr.ps1") {
    Copy-Item "crwlr.ps1" "$INSTALL_DIR\"
    Write-Host "   [OK] CLI wrapper crwlr.ps1 installed" -ForegroundColor Green
}

# Add to PATH
Write-Host "[INSTALL] Adding to system PATH..." -ForegroundColor Yellow
$currentPath = [Environment]::GetEnvironmentVariable("Path", "Machine")
if ($currentPath -notlike "*$INSTALL_DIR*") {
    [Environment]::SetEnvironmentVariable("Path", "$currentPath;$INSTALL_DIR", "Machine")
    $env:Path = [Environment]::GetEnvironmentVariable("Path", "Machine")
}

# Create desktop shortcut
Write-Host "[INSTALL] Creating desktop shortcut..." -ForegroundColor Yellow
$WshShell = New-Object -ComObject WScript.Shell
$Shortcut = $WshShell.CreateShortcut("$DESKTOP\$APP_NAME.lnk")
$Shortcut.TargetPath = "$INSTALL_DIR\$exeName"
$Shortcut.WorkingDirectory = $INSTALL_DIR
$Shortcut.Description = "Crusty-Crawler System Monitoring"
$Shortcut.Save()

# Create Start Menu shortcut
Write-Host "[INSTALL] Creating Start Menu shortcut..." -ForegroundColor Yellow
$startMenuPath = "$env:ProgramData\Microsoft\Windows\Start Menu\Programs"
$StartShortcut = $WshShell.CreateShortcut("$startMenuPath\$APP_NAME.lnk")
$StartShortcut.TargetPath = "$INSTALL_DIR\$exeName"
$StartShortcut.WorkingDirectory = $INSTALL_DIR
$StartShortcut.Description = "Crusty-Crawler System Monitoring"
$StartShortcut.Save()

# Add Windows Firewall rule
Write-Host "[INSTALL] Adding Windows Firewall rule..." -ForegroundColor Yellow
try {
    New-NetFirewallRule -DisplayName "Crusty-Crawler" -Direction Inbound -Program "$INSTALL_DIR\$exeName" -Action Allow -Profile Any -ErrorAction SilentlyContinue | Out-Null
    Write-Host "   [OK] Firewall rule added" -ForegroundColor Green
} catch {
    Write-Host "   [WARN] Could not add firewall rule (may already exist)" -ForegroundColor Yellow
}

Write-Host ""
Write-Host "[SUCCESS] Installation completed successfully!" -ForegroundColor Green
Write-Host ""
Write-Host "Installation directory: $INSTALL_DIR" -ForegroundColor Cyan
Write-Host "Desktop shortcut created" -ForegroundColor Cyan
Write-Host "Start Menu shortcut created" -ForegroundColor Cyan
Write-Host "Mode: $(if ($MODE -eq "cli") { "CLI" } else { "GUI" })" -ForegroundColor Cyan
Write-Host ""
Write-Host "To run the application:" -ForegroundColor Yellow
if ($MODE -eq "cli") {
    Write-Host "  - CLI Mode: crusty-crawler.exe --cli" -ForegroundColor White
    Write-Host "  - Daemon Mode: crusty-crawler.exe --daemon" -ForegroundColor White
    Write-Host "  - GUI Mode: crusty-crawler.exe (no flags)" -ForegroundColor White
} else {
    Write-Host "  - Double-click the desktop shortcut" -ForegroundColor White
    Write-Host "  - Search for Crusty-Crawler in Start Menu" -ForegroundColor White
    Write-Host "  - Run crusty-crawler.exe from command line" -ForegroundColor White
    Write-Host "  - CLI Mode: crusty-crawler.exe --cli" -ForegroundColor White
}
Write-Host ""
Write-Host "Enjoy using Crusty-Crawler!" -ForegroundColor Cyan

# Ask if user wants to run now
if ($MODE -eq "gui") {
    Write-Host ""
    $response = Read-Host "Would you like to launch Crusty-Crawler now? (Y/N)"
    if ($response -eq "Y" -or $response -eq "y") {
        Start-Process "$INSTALL_DIR\$exeName"
    }
} else {
    Write-Host ""
    Write-Host "To start in CLI mode, run: crusty-crawler.exe --cli" -ForegroundColor Yellow
}
