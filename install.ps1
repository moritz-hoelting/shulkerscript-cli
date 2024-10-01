# Error setup
$ErrorActionPreference = "Stop"
Set-StrictMode -Version 3.0
Set-PSDebug -Strict

# Variables
$REPO = "moritz-hoelting/shulkerscript-cli"
$PROGRAM_DISPLAY_NAME = "Shulkerscript CLI"
$LATEST_RELEASE_URL = "https://api.github.com/repos/$REPO/releases/latest"
$BIN_NAME = "shulkerscript"
$CRATE_NAME = "shulkerscript-cli"
$INSTALL_PATH = Join-Path $env:USERPROFILE "AppData\Local\Programs\$BIN_NAME"
$PATH_REGISTRY = "Registry::HKEY_CURRENT_USER\Environment"

function Remove-Old-Version {
    if ( (Test-Path variable:INSTALLED_VERSION) -and ($INSTALL_PATH -notlike "*\.cargo\bin*") ) {
        Write-Host Removing old version at $INSTALL_PATH
        Remove-Item -Path "$INSTALL_PATH" -Force -Recurse
    }
}

# Determine the OS and architecture
$OS = 'windows'
$ARCH = $env:PROCESSOR_ARCHITECTURE

if ($ARCH -eq 'AMD64') {
    $ARCH = 'x86_64'
} elseif ($ARCH -eq 'x86') {
    $ARCH = 'i686'
} else {
    Write-Host "Unsupported architecture: $ARCH" -ForegroundColor Red
    return "Error"
}

# Fetch the latest release data from GitHub
try {
    $LATEST_RELEASE_DATA = Invoke-RestMethod -Uri $LATEST_RELEASE_URL -Method Get
}
catch {
    Write-Host "Failed to fetch latest release data." -ForegroundColor Red
    return "Error"
}

# Get the latest version number
$LATEST_VERSION = $LATEST_RELEASE_DATA.tag_name

# Check if the CLI is already installed and get the current version
if (Get-Command $BIN_NAME -ErrorAction Ignore) {
    $INSTALLED_VERSION = (& $BIN_NAME --version | Select-String -Pattern '\d+\.\d+\.\d+(-(rc|beta|alpha)(\.\d+)?)?' | Select-Object -First 1).Matches.Value
    $CLEAN_LATEST_VERSION = (Write-Output $LATEST_VERSION | Select-String -Pattern '\d+\.\d+\.\d+(-(rc|beta|alpha)(\.\d+)?)?').Matches.Value
    Write-Host "Installed version: v$INSTALLED_VERSION"
    Write-Host "Latest version: v$CLEAN_LATEST_VERSION"

    if ($INSTALLED_VERSION -eq $CLEAN_LATEST_VERSION) {
        Write-Host "$PROGRAM_DISPLAY_NAME is already up to date."
        return
    }
    else {
        Write-Host "A new version is available. Upgrading..."
        $INSTALL_PATH = Split-Path -Parent (Get-Command $BIN_NAME).Path
    }
}
else {
    Write-Host "$PROGRAM_DISPLAY_NAME is not installed. Installing version $LATEST_VERSION..."
}

# Use cargo-binstall if available
if (Get-Command cargo-binstall -ErrorAction SilentlyContinue) {
    Write-Host "cargo-binstall is available. Installing/upgrading using cargo-binstall..."
    cargo-binstall --git "https://github.com/$REPO" --force --locked --no-confirm $CRATE_NAME
    Remove-Old-Version
    return
}

# Get the download url of the latest release
$DOWNLOAD_URL = ($LATEST_RELEASE_DATA.assets | Where-Object {$_.browser_download_url -match "$OS" -and $_.browser_download_url -match "$ARCH"} | Select-Object -First 1).browser_download_url

if ([string]::IsNullOrEmpty($DOWNLOAD_URL)) {
    # if there is no prebuilt binary, try to build from source
    if (Get-Command cargo -ErrorAction SilentlyContinue) {
        Write-Host "No prebuilt binary available for your platform. Building from source..."
        cargo install --force --locked $CRATE_NAME
        Remove-Old-Version
        return
    }
    else {
        Write-Host "No prebuilt binary available for your platform. Please install Rust and Cargo using https://rustup.rs and try again."
        return "Error"
    }
}

# Create a temporary directory
$TEMP = [System.IO.Path]::GetTempPath()
$TEMP_DIR = Join-Path $TEMP (New-Guid).ToString("N")
Remove-Item -Path "$TEMP_DIR" -Recurse -Force -ErrorAction Ignore | Out-Null
New-Item -ItemType Directory -Path $TEMP_DIR | Out-Null

# Download and extract the binary
Invoke-WebRequest -Uri $DOWNLOAD_URL -OutFile "$TEMP_DIR\$BIN_NAME.zip"
Expand-Archive -Path "$TEMP_DIR\$BIN_NAME.zip" -DestinationPath $TEMP_DIR -Force

# Create install location and move binary
New-Item -ItemType Directory -Path $INSTALL_PATH -Force | Out-Null
Move-Item -Path "$TEMP_DIR\$BIN_NAME.exe" -Destination "$INSTALL_PATH\$BIN_NAME.exe" -Force

# Remove temp dir
Remove-Item -Path "$TEMP_DIR" -Recurse -Force -ErrorAction Ignore | Out-Null

# Add binary to PATH
$REGEX_INSTALL_PATH = [regex]::Escape($INSTALL_PATH)
$ARR_PATH = $env:Path -split ';' | Where-Object {$_ -match "^$REGEX_INSTALL_PATH\\?"}
if (-not $ARR_PATH) {
    Write-Host "Not found in current PATH, adding..."
    $OLD_PATH = (Get-ItemProperty -Path "$PATH_REGISTRY" -Name PATH).path
    $NEW_PATH = "$OLD_PATH;$INSTALL_PATH"
    Set-ItemProperty -Path "$PATH_REGISTRY" -Name PATH -Value $NEW_PATH
    $env:PATH="$NEW_PATH"
}

Write-Host "$PROGRAM_DISPLAY_NAME has been successfully installed/upgraded to version $LATEST_VERSION."
