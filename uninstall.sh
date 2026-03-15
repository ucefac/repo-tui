#!/bin/bash
set -e

INSTALL_DIR="$HOME/.config/repo-tui"

echo "🗑️  Uninstalling repo-tui..."

if [[ ! -d "$INSTALL_DIR" ]]; then
    echo "ℹ️  repo-tui is not installed."
    exit 0
fi

rm -rf "$INSTALL_DIR"

echo "✅ repo-tui uninstalled successfully!"
echo ""
echo "📝 Note: ~/.zshrc entries for repo-tui were not removed."
echo "   You can manually remove the following lines from ~/.zshrc:"
echo ""
echo "   # repo-tui Shell Integration"
echo '   [[ -f "$HOME/.config/repo-tui/zsh/bin/repo-tui.zsh" ]] && source "$HOME/.config/repo-tui/zsh/bin/repo-tui.zsh"'
echo ""
echo "   Then run 'source ~/.zshrc' or restart your terminal."
