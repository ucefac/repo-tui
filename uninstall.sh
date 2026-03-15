#!/bin/bash
set -e

INSTALL_DIR="$HOME/.config/repotui"

echo "🗑️  Uninstalling repotui..."

if [[ ! -d "$INSTALL_DIR" ]]; then
    echo "ℹ️  repotui is not installed."
    exit 0
fi

rm -rf "$INSTALL_DIR"

echo "✅ repotui uninstalled successfully!"
echo ""
echo "📝 Note: ~/.zshrc entries for repotui were not removed."
echo "   You can manually remove the following lines from ~/.zshrc:"
echo ""
echo "   # repotui Shell Integration"
echo '   [[ -f "$HOME/.config/repotui/zsh/repotui.zsh" ]] && source "$HOME/.config/repotui/zsh/repotui.zsh"'
echo ""
echo "   Then run 'source ~/.zshrc' or restart your terminal."
