#!/bin/bash
set -e

# Configuration
REPO="ucefac/repo-tui"
INSTALL_DIR="$HOME/.config/repo-tui"
BIN_DIR="$INSTALL_DIR/bin"
ZSH_DIR="$INSTALL_DIR/zsh"
ZSH_CONFIG="$ZSH_DIR/repo-tui.zsh"

echo "🚀 Installing repo-tui..."

# Architecture check
ARCH=$(uname -m)
if [[ "$ARCH" != "arm64" && "$ARCH" != "aarch64" ]]; then
    echo "❌ Error: Only macOS ARM64 (aarch64) is supported."
    echo "   Your architecture: $ARCH"
    exit 1
fi

echo "✓ Architecture check passed: $ARCH"

# Network check
if ! curl -s --head https://github.com &>/dev/null; then
    echo "❌ Error: Cannot reach github.com, please check your network connection."
    exit 1
fi

echo "✓ Network check passed"

# Get latest version from GitHub API
echo "📦 Fetching latest version..."
LATEST=$(curl -s https://api.github.com/repos/$REPO/releases/latest)
VERSION=$(echo "$LATEST" | grep '"tag_name"' | cut -d'"' -f4)
if [[ -z "$VERSION" ]]; then
    echo "❌ Error: Failed to get latest version from GitHub API"
    exit 1
fi

echo "✓ Latest version: $VERSION"

# Download
DOWNLOAD_URL="https://github.com/$REPO/releases/download/$VERSION/repo-tui-aarch64-apple-darwin.tar.gz"
TMP_FILE="/tmp/repo-tui-$VERSION.tar.gz"

echo "📥 Downloading from GitHub..."
if ! curl -L "$DOWNLOAD_URL" -o "$TMP_FILE" 2>/dev/null; then
    echo "❌ Error: Failed to download release file"
    exit 1
fi

echo "✓ Download completed"

# Install
echo "📦 Installing..."
mkdir -p "$BIN_DIR" "$ZSH_DIR"
tar -xzf "$TMP_FILE" -C "$BIN_DIR"
chmod +x "$BIN_DIR/repo-tui"

echo "✓ Binary installed to $BIN_DIR"

# Create zsh config
cat > "$ZSH_CONFIG" << EOF
# repo-tui Zsh Integration - DO NOT EDIT MANUALLY
export REPO_TUI_BIN_DIR="$HOME/.config/repo-tui/bin"
export PATH="$REPO_TUI_BIN_DIR:$PATH"
EOF

echo "✓ Zsh configuration created"

# Add to .zshrc if not present
ZSHRC="$HOME/.zshrc"
SOURCE_LINE='[[ -f "$HOME/.config/repo-tui/zsh/repo-tui.zsh" ]] && source "$HOME/.config/repo-tui/zsh/repo-tui.zsh"'

if [[ ! -f "$ZSHRC" ]]; then
    touch "$ZSHRC"
fi

if ! grep -q "repo-tui.zsh" "$ZSHRC" 2>/dev/null; then
    echo "" >> "$ZSHRC"
    echo "# repo-tui Shell Integration" >> "$ZSHRC"
    echo "$SOURCE_LINE" >> "$ZSHRC"
    echo "✓ Added repo-tui to PATH in ~/.zshrc"
else
    echo "✓ repo-tui already configured in ~/.zshrc"
fi

# Cleanup
rm "$TMP_FILE"

echo ""
echo "✅ repo-tui $VERSION installed successfully!"
echo ""
echo "📝 Next steps:"
echo "   Run 'source ~/.zshrc' or restart your terminal, then run 'repo-tui'"
