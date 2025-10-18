# Crusty-Crawler CLI wrapper script (Windows)
# Provides convenient commands for managing the application

param(
    [Parameter(Position=0)]
    [string]$Command
)

$BINARY = "crusty-crawler.exe"
$INSTALL_DIR = "$env:ProgramFiles\Crusty-Crawler"

function Show-Help {
    Write-Host "Crusty-Crawler CLI Management Tool" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "Usage: crwlr <command>" -ForegroundColor Yellow
    Write-Host ""
    Write-Host "Run Modes:" -ForegroundColor Green
    Write-Host "  cli        Run interactive CLI"
    Write-Host "  daemon     Run in daemon mode (foreground)"
    Write-Host "  gui        Run GUI interface"
    Write-Host "  status     Show if process is running"
    Write-Host ""
    Write-Host "Process Management:" -ForegroundColor Green
    Write-Host "  start      Start as background process"
    Write-Host "  stop       Stop all running instances"
    Write-Host ""
    Write-Host "Other:" -ForegroundColor Green
    Write-Host "  help       Show this help message"
}

switch ($Command) {
    "start" {
        Write-Host "[START] Starting Crusty-Crawler..." -ForegroundColor Green
        Start-Process -FilePath "$INSTALL_DIR\$BINARY" -ArgumentList "--daemon" -WindowStyle Hidden
        Write-Host "[OK] Started in background" -ForegroundColor Green
    }
    "stop" {
        Write-Host "[STOP] Stopping Crusty-Crawler..." -ForegroundColor Yellow
        Get-Process -Name "crusty-crawler" -ErrorAction SilentlyContinue | Stop-Process -Force
        Write-Host "[OK] Stopped" -ForegroundColor Green
    }
    "status" {
        $process = Get-Process -Name "crusty-crawler" -ErrorAction SilentlyContinue
        if ($process) {
            Write-Host "[RUNNING] Crusty-Crawler is running" -ForegroundColor Green
            Write-Host "   PID: $($process.Id)"
            Write-Host "   Memory: $([math]::Round($process.WorkingSet64 / 1MB, 2)) MB"
        } else {
            Write-Host "[STOPPED] Crusty-Crawler is not running" -ForegroundColor Red
        }
    }
    "cli" {
        Write-Host "[CLI] Starting CLI interface..." -ForegroundColor Cyan
        & "$INSTALL_DIR\$BINARY" --cli
    }
    "daemon" {
        Write-Host "[DAEMON] Starting in daemon mode (Ctrl+C to stop)..." -ForegroundColor Cyan
        & "$INSTALL_DIR\$BINARY" --daemon
    }
    "gui" {
        Write-Host "[GUI] Starting GUI interface..." -ForegroundColor Cyan
        Start-Process -FilePath "$INSTALL_DIR\$BINARY"
    }
    "help" {
        Show-Help
    }
    default {
        if ([string]::IsNullOrEmpty($Command)) {
            Show-Help
        } else {
            Write-Host "Unknown command: $Command" -ForegroundColor Red
            Write-Host "Run crwlr help for usage information" -ForegroundColor Yellow
        }
    }
}
