#!/usr/bin/env pwsh
# GhostFS Installer for Windows
# Usage: irm https://raw.githubusercontent.com/shejanmahamud/ghostfs/main/install.ps1 | iex

$ErrorActionPreference = "Stop"

$repo = "shejanmahamud/ghostfs"
$binaryName = "ghost.exe"
$installDir = "$env:USERPROFILE\.ghostfs\bin"

Write-Host ""
Write-Host "  👻 GhostFS Installer" -ForegroundColor Cyan
Write-Host "  =====================" -ForegroundColor DarkCyan
Write-Host ""

# Detect architecture
$arch = if ([System.Environment]::Is64BitOperatingSystem) { "x86_64" } else { "i686" }
$target = "$arch-pc-windows-msvc"
$assetName = "ghost-$target.zip"

# Get latest release
Write-Host "  Fetching latest release..." -ForegroundColor DarkGray
try {
    $release = Invoke-RestMethod -Uri "https://api.github.com/repos/$repo/releases/latest" -Headers @{ "User-Agent" = "ghostfs-installer" }
    $version = $release.tag_name
    $asset = $release.assets | Where-Object { $_.name -eq $assetName }

    if (-not $asset) {
        Write-Host "  Error: No binary found for $target" -ForegroundColor Red
        Write-Host "  Available assets:" -ForegroundColor Yellow
        $release.assets | ForEach-Object { Write-Host "    - $($_.name)" -ForegroundColor DarkGray }
        exit 1
    }

    $downloadUrl = $asset.browser_download_url
} catch {
    Write-Host "  Error: Could not fetch release info from GitHub." -ForegroundColor Red
    Write-Host "  $_" -ForegroundColor DarkGray
    exit 1
}

Write-Host "  Version:  $version" -ForegroundColor Green
Write-Host "  Platform: $target" -ForegroundColor DarkGray
Write-Host ""

# Create install directory
if (-not (Test-Path $installDir)) {
    New-Item -ItemType Directory -Path $installDir -Force | Out-Null
}

# Download
$tempZip = Join-Path $env:TEMP "ghostfs-download.zip"
Write-Host "  Downloading $assetName..." -ForegroundColor DarkGray
Invoke-WebRequest -Uri $downloadUrl -OutFile $tempZip -UseBasicParsing

# Extract
Write-Host "  Extracting..." -ForegroundColor DarkGray
Expand-Archive -Path $tempZip -DestinationPath $installDir -Force
Remove-Item $tempZip -Force

# Verify
$binaryPath = Join-Path $installDir $binaryName
if (-not (Test-Path $binaryPath)) {
    Write-Host "  Error: Binary not found after extraction" -ForegroundColor Red
    exit 1
}

# Add to PATH
$userPath = [System.Environment]::GetEnvironmentVariable("Path", "User")
if ($userPath -notlike "*$installDir*") {
    [System.Environment]::SetEnvironmentVariable("Path", "$installDir;$userPath", "User")
    Write-Host "  Added $installDir to PATH" -ForegroundColor DarkGray
}

# Update current session
$env:Path = "$installDir;$env:Path"

Write-Host ""
Write-Host "  ✅ GhostFS $version installed successfully!" -ForegroundColor Green
Write-Host ""
Write-Host "  Location: $binaryPath" -ForegroundColor DarkGray
Write-Host ""
Write-Host "  Get started:" -ForegroundColor White
Write-Host "    ghost init" -ForegroundColor Cyan
Write-Host "    ghost add react" -ForegroundColor Cyan
Write-Host "    ghost install" -ForegroundColor Cyan
Write-Host ""
Write-Host "  Run 'ghost --help' for all commands." -ForegroundColor DarkGray
Write-Host ""
