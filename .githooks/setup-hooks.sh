#!/bin/bash
# Setup Git Hooks for Project Chimera
# Run this script to install security git hooks

set -e

HOOKS_DIR=".githooks"
GIT_HOOKS_DIR=".git/hooks"

echo "🔧 Setting up Project Chimera Git Hooks..."
echo ""

# Check if we're in a git repository
if [ ! -d ".git" ]; then
    echo "❌ Error: Not in a git repository"
    echo "Please run this script from the project root directory"
    exit 1
fi

# Create hooks directory if it doesn't exist
mkdir -p "$GIT_HOOKS_DIR"

# Install pre-commit hook
if [ -f "$HOOKS_DIR/pre-commit" ]; then
    echo "📋 Installing pre-commit hook..."
    cp "$HOOKS_DIR/pre-commit" "$GIT_HOOKS_DIR/pre-commit"
    chmod +x "$GIT_HOOKS_DIR/pre-commit"
    echo "   ✅ pre-commit hook installed"
else
    echo "   ⚠️  pre-commit hook not found in $HOOKS_DIR"
fi

echo ""
echo "✅ Git hooks setup complete!"
echo ""
echo "Installed hooks:"
echo "  - pre-commit: Prevents accidental commit of secrets"
echo ""
echo "To bypass hooks (NOT RECOMMENDED):"
echo "  git commit --no-verify"
echo ""
echo "To uninstall hooks:"
echo "  rm .git/hooks/pre-commit"
echo ""
