#!/bin/bash
# Crusty-Crawler CLI wrapper script
# Provides convenient commands for managing the service

BINARY="/usr/local/bin/crusty-crawler"
SERVICE="crusty-crawler.service"

case "$1" in
    start)
        echo "[START] Starting Crusty-Crawler service..."
        sudo systemctl start $SERVICE
        sudo systemctl status $SERVICE --no-pager
        ;;
    stop)
        echo "[STOP] Stopping Crusty-Crawler service..."
        sudo systemctl stop $SERVICE
        ;;
    restart)
        echo "[RESTART] Restarting Crusty-Crawler service..."
        sudo systemctl restart $SERVICE
        sudo systemctl status $SERVICE --no-pager
        ;;
    status)
        sudo systemctl status $SERVICE
        ;;
    enable)
        echo "[ENABLE] Enabling Crusty-Crawler to start on boot..."
        sudo systemctl enable $SERVICE
        ;;
    disable)
        echo "[DISABLE] Disabling Crusty-Crawler auto-start..."
        sudo systemctl disable $SERVICE
        ;;
    logs)
        echo "[LOGS] Showing logs (Ctrl+C to exit)..."
        sudo journalctl -u $SERVICE -f
        ;;
    cli)
        echo "[CLI] Starting CLI interface..."
        $BINARY --cli
        ;;
    daemon)
        echo "[DAEMON] Starting in daemon mode (Ctrl+C to stop)..."
        $BINARY --daemon
        ;;
    gui)
        echo "[GUI] Starting GUI interface..."
        $BINARY
        ;;
    help|--help|-h)
        echo "Crusty-Crawler CLI Management Tool"
        echo ""
        echo "Usage: crwlr <command>"
        echo ""
        echo "Service Management:"
        echo "  start      Start the service"
        echo "  stop       Stop the service"
        echo "  restart    Restart the service"
        echo "  status     Show service status"
        echo "  enable     Enable auto-start on boot"
        echo "  disable    Disable auto-start"
        echo "  logs       Show service logs (follows)"
        echo ""
        echo "Run Modes:"
        echo "  cli        Run interactive CLI"
        echo "  daemon     Run in daemon mode (foreground)"
        echo "  gui        Run GUI interface"
        echo ""
        echo "Other:"
        echo "  help       Show this help message"
        ;;
    *)
        echo "Unknown command: $1"
        echo "Run 'crwlr help' for usage information"
        exit 1
        ;;
esac
