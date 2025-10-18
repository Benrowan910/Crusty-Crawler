#!/bin/bash
# Installation script for Crusty-Crawler
# Installs the application to /opt/crusty-crawler and creates systemd service

set -e

APP_NAME="crusty-crawler"
INSTALL_DIR="/opt/${APP_NAME}"
BIN_DIR="/usr/local/bin"
SERVICE_DIR="/etc/systemd/system"
SERVICE_NAME="${APP_NAME}.service"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

echo -e "${CYAN}Crusty-Crawler Installation Script${NC}"
echo ""

# Check if running as root
if [ "$EUID" -ne 0 ]; then 
    echo -e "${RED}[ERROR] Please run as root (use sudo)${NC}"
    exit 1
fi

# Ask user for installation mode
echo -e "${YELLOW}Installation Mode Selection${NC}"
echo "1. GUI Mode (Desktop application with graphical interface)"
echo "2. CLI Mode (Command-line interface for servers)"
echo ""
read -p "Select installation mode (1 or 2): " INSTALL_MODE

case "$INSTALL_MODE" in
    1)
        MODE="gui"
        echo -e "${GREEN}Installing GUI mode...${NC}"
        ;;
    2)
        MODE="cli"
        echo -e "${GREEN}Installing CLI mode...${NC}"
        ;;
    *)
        echo -e "${RED}Invalid selection. Defaulting to GUI mode.${NC}"
        MODE="gui"
        ;;
esac

echo ""
echo -e "${YELLOW}[INSTALL] Installing Crusty-Crawler...${NC}"

# Create installation directory
echo -e "${YELLOW}[CREATE] Creating installation directory...${NC}"
mkdir -p "${INSTALL_DIR}"

# Copy binary
echo -e "${YELLOW}[COPY] Installing binary...${NC}"
if [ -f "${APP_NAME}" ]; then
    cp "${APP_NAME}" "${INSTALL_DIR}/"
    chmod +x "${INSTALL_DIR}/${APP_NAME}"
elif [ -f "${APP_NAME}.exe" ]; then
    echo -e "${RED}[ERROR] This appears to be a Windows binary. Please use the Windows installer.${NC}"
    exit 1
else
    echo -e "${RED}[ERROR] Binary not found. Make sure you're in the correct directory.${NC}"
    exit 1
fi

# Copy assets and public directory
echo -e "${YELLOW}[COPY] Installing assets...${NC}"
if [ -d "Assets" ]; then
    cp -r Assets "${INSTALL_DIR}/"
fi
if [ -d "public" ]; then
    cp -r public "${INSTALL_DIR}/"
fi

# Create symlink in /usr/local/bin
echo -e "${YELLOW}[LINK] Creating symlinks...${NC}"
ln -sf "${INSTALL_DIR}/${APP_NAME}" "${BIN_DIR}/${APP_NAME}"

# Install wrapper script if available
if [ -f "crwlr.sh" ]; then
    cp crwlr.sh "${INSTALL_DIR}/"
    chmod +x "${INSTALL_DIR}/crwlr.sh"
    ln -sf "${INSTALL_DIR}/crwlr.sh" "${BIN_DIR}/crwlr"
    echo -e "  [OK] CLI wrapper 'crwlr' installed"
fi

# Create systemd service file
echo -e "${YELLOW}[SERVICE] Creating systemd service...${NC}"
if [ "$MODE" = "cli" ]; then
    # CLI mode - run with --daemon flag
    cat > "${SERVICE_DIR}/${SERVICE_NAME}" << EOF
[Unit]
Description=Crusty-Crawler System Monitoring Service (CLI Mode)
After=network.target

[Service]
Type=simple
User=root
WorkingDirectory=${INSTALL_DIR}
ExecStart=${INSTALL_DIR}/${APP_NAME} --daemon
Restart=on-failure
RestartSec=10
StandardOutput=journal
StandardError=journal

[Install]
WantedBy=multi-user.target
EOF
else
    # GUI mode - standard service
    cat > "${SERVICE_DIR}/${SERVICE_NAME}" << EOF
[Unit]
Description=Crusty-Crawler System Monitoring Service (GUI Mode)
After=network.target graphical.target

[Service]
Type=simple
User=root
WorkingDirectory=${INSTALL_DIR}
ExecStart=${INSTALL_DIR}/${APP_NAME}
Restart=on-failure
RestartSec=10
StandardOutput=journal
StandardError=journal
Environment=DISPLAY=:0

[Install]
WantedBy=multi-user.target
EOF
fi

# Set proper permissions
chmod 644 "${SERVICE_DIR}/${SERVICE_NAME}"

# Reload systemd
echo -e "${YELLOW}[RELOAD] Reloading systemd...${NC}"
systemctl daemon-reload

echo ""
echo -e "${GREEN}[SUCCESS] Installation completed successfully!${NC}"
echo ""
echo -e "${CYAN}[INFO] Installation directory: ${INSTALL_DIR}${NC}"
echo -e "${CYAN}[INFO] Binary symlink: ${BIN_DIR}/${APP_NAME}${NC}"
echo -e "${CYAN}[INFO] Service file: ${SERVICE_DIR}/${SERVICE_NAME}${NC}"
echo -e "${CYAN}[INFO] Mode: $([ "$MODE" = "cli" ] && echo "CLI" || echo "GUI")${NC}"
echo ""
echo -e "${YELLOW}To manage the service:${NC}"
echo -e "  Start:   ${GREEN}sudo systemctl start ${SERVICE_NAME}${NC}"
echo -e "  Stop:    ${GREEN}sudo systemctl stop ${SERVICE_NAME}${NC}"
echo -e "  Enable:  ${GREEN}sudo systemctl enable ${SERVICE_NAME}${NC}"
echo -e "  Status:  ${GREEN}sudo systemctl status ${SERVICE_NAME}${NC}"
echo ""
if [ "$MODE" = "cli" ]; then
    echo -e "${YELLOW}CLI Mode Commands:${NC}"
    echo -e "  Interactive:  ${GREEN}${APP_NAME}${NC}"
    echo -e "  With flag:    ${GREEN}${APP_NAME} --cli${NC}"
    echo -e "  Daemon mode:  ${GREEN}${APP_NAME} --daemon${NC}"
else
    echo -e "${YELLOW}To run manually:${NC}"
    echo -e "  GUI:  ${GREEN}${APP_NAME}${NC}"
    echo -e "  CLI:  ${GREEN}${APP_NAME} --cli${NC}"
fi
echo ""
echo -e "${CYAN}Enjoy using Crusty-Crawler!${NC}"
