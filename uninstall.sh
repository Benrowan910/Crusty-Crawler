#!/bin/bash
# Uninstallation script for Crusty-Crawler

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

echo -e "${CYAN}Crusty-Crawler Uninstallation Script${NC}"
echo ""

# Check if running as root
if [ "$EUID" -ne 0 ]; then 
    echo -e "${RED}[ERROR] Please run as root (use sudo)${NC}"
    exit 1
fi

echo -e "${YELLOW}[UNINSTALL] Uninstalling Crusty-Crawler...${NC}"

# Stop and disable service if running
if systemctl is-active --quiet "${SERVICE_NAME}"; then
    echo -e "${YELLOW}[STOP] Stopping service...${NC}"
    systemctl stop "${SERVICE_NAME}"
fi

if systemctl is-enabled --quiet "${SERVICE_NAME}" 2>/dev/null; then
    echo -e "${YELLOW}[DISABLE] Disabling service...${NC}"
    systemctl disable "${SERVICE_NAME}"
fi

# Remove service file
if [ -f "${SERVICE_DIR}/${SERVICE_NAME}" ]; then
    echo -e "${YELLOW}[REMOVE] Removing service file...${NC}"
    rm -f "${SERVICE_DIR}/${SERVICE_NAME}"
    systemctl daemon-reload
fi

# Remove symlink
if [ -L "${BIN_DIR}/${APP_NAME}" ]; then
    echo -e "${YELLOW}[REMOVE] Removing symlink...${NC}"
    rm -f "${BIN_DIR}/${APP_NAME}"
fi

# Remove installation directory
if [ -d "${INSTALL_DIR}" ]; then
    echo -e "${YELLOW}[REMOVE] Removing installation directory...${NC}"
    rm -rf "${INSTALL_DIR}"
fi

echo ""
echo -e "${GREEN}[SUCCESS] Uninstallation completed successfully!${NC}"
echo -e "${CYAN}Crusty-Crawler has been removed from your system.${NC}"
