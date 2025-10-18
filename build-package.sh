#!/bin/bash
# Build script for Crusty-Crawler
# Creates a distributable tar package

set -e

VERSION="0.1.0"
APP_NAME="crusty-crawler"
PACKAGE_NAME="${APP_NAME}-${VERSION}"

echo "[BUILD] Building Crusty-Crawler v${VERSION}..."

# Build the release version
echo "[COMPILE] Compiling release build..."
cargo build --release

# Create package directory
echo "[CREATE] Creating package directory..."
mkdir -p "dist/${PACKAGE_NAME}"

# Copy binary
echo "[COPY] Copying binary..."
cp "target/release/RustSystemChecker" "dist/${PACKAGE_NAME}/${APP_NAME}" 2>/dev/null || \
cp "target/release/RustSystemChecker.exe" "dist/${PACKAGE_NAME}/${APP_NAME}.exe" 2>/dev/null || true

# Copy assets and public directory
echo "[COPY] Copying assets..."
cp -r Assets "dist/${PACKAGE_NAME}/"
cp -r public "dist/${PACKAGE_NAME}/"

# Copy license and readme
echo "[COPY] Copying documentation..."
cp LICENSE "dist/${PACKAGE_NAME}/" 2>/dev/null || echo "No LICENSE file found"
cp README.md "dist/${PACKAGE_NAME}/" 2>/dev/null || echo "No README.md file found"

# Copy install scripts
echo "[COPY] Copying install scripts..."
cp install.sh "dist/${PACKAGE_NAME}/" 2>/dev/null || echo "Warning: install.sh not found"
cp install.ps1 "dist/${PACKAGE_NAME}/" 2>/dev/null || echo "Warning: install.ps1 not found"
cp uninstall.sh "dist/${PACKAGE_NAME}/" 2>/dev/null || echo "Warning: uninstall.sh not found"
cp uninstall.ps1 "dist/${PACKAGE_NAME}/" 2>/dev/null || echo "Warning: uninstall.ps1 not found"
cp crwlr.sh "dist/${PACKAGE_NAME}/" 2>/dev/null || echo "Warning: crwlr.sh not found"
cp crwlr.ps1 "dist/${PACKAGE_NAME}/" 2>/dev/null || echo "Warning: crwlr.ps1 not found"

# Create tar package
echo "[PACKAGE] Creating tar.gz package..."
cd dist
tar -czf "${PACKAGE_NAME}.tar.gz" "${PACKAGE_NAME}"
cd ..

# Create checksums
echo "[CHECKSUM] Generating checksums..."
cd dist
sha256sum "${PACKAGE_NAME}.tar.gz" > "${PACKAGE_NAME}.tar.gz.sha256"
cd ..

echo ""
echo "[SUCCESS] Package created successfully!"
echo "[INFO] Location: dist/${PACKAGE_NAME}.tar.gz"
echo "[INFO] Checksum: dist/${PACKAGE_NAME}.tar.gz.sha256"
echo ""
echo "To install, extract the tar.gz and run:"
echo "  tar -xzf ${PACKAGE_NAME}.tar.gz"
echo "  cd ${PACKAGE_NAME}"
echo "  sudo ./install.sh"
