#!/bin/bash

# Error setup
set -euo pipefail

# Variables
REPO="moritz-hoelting/shulkerscript-cli"
PROGRAM_DISPLAY_NAME="Shulkerscript CLI"
LATEST_RELEASE_URL="https://api.github.com/repos/$REPO/releases/latest"
BIN_NAME="shulkerscript"
INSTALL_PATH="$HOME/bin/$BIN_NAME"

function removeOldVersion() {
    if [ ! -z ${INSTALLED_VERSION+x} ] && [[ $INSTALL_PATH != *"/.cargo/bin/"* ]]; then
        rm -f $INSTALL_PATH
        hash -d $BIN_NAME &> /dev/null || true
    fi
}

# Determine the OS and architecture
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

# Fetch the latest release data from GitHub
LATEST_RELEASE_DATA=$(curl -s $LATEST_RELEASE_URL)

# Get the latest version number
LATEST_VERSION=$(echo "$LATEST_RELEASE_DATA" | grep 'tag_name' | cut -d '"' -f 4)

# Check if the CLI is already installed and get the current version
if which $BIN_NAME &> /dev/null; then
    INSTALLED_VERSION=$($BIN_NAME --version | grep -m 1 -oE '[0-9]+\.[0-9]+\.[0-9]+(-(rc|beta|alpha)(\.\d+)?)?')
    CLEAN_LATEST_VERSION=$(echo $LATEST_VERSION | grep -oE '[0-9]+\.[0-9]+\.[0-9]+(-(rc|beta|alpha)(\.\d+)?)?')
    echo "Installed version: v$INSTALLED_VERSION"
    echo "Latest version: v$CLEAN_LATEST_VERSION"

    if [ "$INSTALLED_VERSION" == "$CLEAN_LATEST_VERSION" ]; then
        echo "$PROGRAM_DISPLAY_NAME is already up to date."
        exit 0
    else
        echo "A new version is available. Upgrading..."
        INSTALL_PATH=$(which $BIN_NAME)
    fi
else
    echo "$PROGRAM_DISPLAY_NAME is not installed. Installing version $LATEST_VERSION..."
fi

# Use cargo-binstall if available
if which cargo-binstall &> /dev/null; then
    echo "Found cargo-binstall. Installing/upgrading using cargo-binstall..."
    cargo-binstall --git "https://github.com/$REPO" --force --locked --no-confirm $BIN_NAME
    
    # Remove old version
    removeOldVersion
    exit 0
fi

# Get the download url of the latest release
DOWNLOAD_URL=$(echo "$LATEST_RELEASE_DATA" | awk "/browser_download_url/ && /$OS/ && /$ARCH/" | cut -d '"' -f 4)

if [ -z "$DOWNLOAD_URL" ]; then
    # if there is no prebuilt binary, try to build from source
    if which cargo &> /dev/null; then
        echo "No prebuilt binary available for your platform. Building from source..."
        cargo install --git "https://github.com/$REPO" --force --locked
        removeOldVersion
        exit 0
    else
        echo "No prebuilt binary available for your platform. Please install Rust and Cargo using https://rustup.rs and try again."
        exit 1
    fi
fi

if [[ "$DOWNLOAD_URL" == *"tar.gz" ]]; then
    ARCHIVE_TYPE="tar.gz"
elif [[ "$DOWNLOAD_URL" == *"zip" ]]; then
    ARCHIVE_TYPE="zip"
else
    echo "Unsupported archive type."
    exit 1
fi

# Create a temporary directory
TEMP_DIR=$(mktemp -d)

# Download and extract the binary
curl -L -s $DOWNLOAD_URL -o $TEMP_DIR/$BIN_NAME.$ARCHIVE_TYPE
if [[ "$ARCHIVE_TYPE" == "tar.gz" ]]; then
    tar -xzf $TEMP_DIR/$BIN_NAME.$ARCHIVE_TYPE -C $TEMP_DIR
else
    unzip $TEMP_DIR/$BIN_NAME.$ARCHIVE_TYPE -d $TEMP_DIR
fi

chmod +x "$TEMP_DIR/$BIN_NAME"
mv "$TEMP_DIR/$BIN_NAME" "$INSTALL_PATH"

echo "$PROGRAM_DISPLAY_NAME has been successfully installed/upgraded to version $LATEST_VERSION."
