# Build script for Crusty-Crawler (Windows PowerShell)
# Creates a distributable zip package

$ErrorActionPreference = "Stop"

$VERSION = "0.1.0"
$APP_NAME = "crusty-crawler"
$PACKAGE_NAME = "$APP_NAME-$VERSION"

Write-Host "🦀 Building Crusty-Crawler v$VERSION..." -ForegroundColor Cyan

# Build the release version
Write-Host "📦 Compiling release build..." -ForegroundColor Yellow
cargo build --release

# Create package directory
Write-Host "📁 Creating package directory..." -ForegroundColor Yellow
$distPath = "dist\$PACKAGE_NAME"
if (Test-Path $distPath) {
    Remove-Item -Recurse -Force $distPath
}
New-Item -ItemType Directory -Path $distPath | Out-Null

# Copy binary
Write-Host "📋 Copying binary..." -ForegroundColor Yellow
Copy-Item "target\release\RustSystemChecker.exe" "$distPath\$APP_NAME.exe"

# Copy assets and public directory
Write-Host "🎨 Copying assets..." -ForegroundColor Yellow
Copy-Item -Recurse "Assets" "$distPath\"
Copy-Item -Recurse "public" "$distPath\"

# Copy license and readme
Write-Host "📄 Copying documentation..." -ForegroundColor Yellow
if (Test-Path "LICENSE") {
    Copy-Item "LICENSE" "$distPath\"
}
if (Test-Path "README.md") {
    Copy-Item "README.md" "$distPath\"
}

# Copy install scripts
Write-Host "🔧 Copying install scripts..." -ForegroundColor Yellow
if (Test-Path "install.sh") { Copy-Item "install.sh" "$distPath\" }
if (Test-Path "install.ps1") { Copy-Item "install.ps1" "$distPath\" }
if (Test-Path "uninstall.sh") { Copy-Item "uninstall.sh" "$distPath\" }
if (Test-Path "uninstall.ps1") { Copy-Item "uninstall.ps1" "$distPath\" }
if (Test-Path "crwlr.sh") { Copy-Item "crwlr.sh" "$distPath\" }
if (Test-Path "crwlr.ps1") { Copy-Item "crwlr.ps1" "$distPath\" }

# Create zip package
Write-Host "📦 Creating zip package..." -ForegroundColor Yellow
$zipPath = "dist\$PACKAGE_NAME.zip"
if (Test-Path $zipPath) {
    Remove-Item $zipPath
}
Compress-Archive -Path $distPath -DestinationPath $zipPath

# Create checksums
Write-Host "🔐 Generating checksums..." -ForegroundColor Yellow
$hash = Get-FileHash $zipPath -Algorithm SHA256
$hash.Hash + "  $PACKAGE_NAME.zip" | Out-File "dist\$PACKAGE_NAME.zip.sha256"

Write-Host ""
Write-Host "✅ Package created successfully!" -ForegroundColor Green
Write-Host "📍 Location: dist\$PACKAGE_NAME.zip" -ForegroundColor Green
Write-Host "🔐 Checksum: dist\$PACKAGE_NAME.zip.sha256" -ForegroundColor Green
Write-Host ""
Write-Host "To install, extract the zip and run as Administrator:" -ForegroundColor Cyan
Write-Host "  Expand-Archive $PACKAGE_NAME.zip" -ForegroundColor White
Write-Host "  cd $PACKAGE_NAME" -ForegroundColor White
Write-Host "  .\install.ps1" -ForegroundColor White
