#!/bin/bash

# Dump all config files for a specific Tauri app bundle
# Usage: ./dump-config.sh <bundle-identifier>
# Example: ./dump-config.sh app.dictara.dev

BUNDLE_ID="${1:-app.dictara.dev}"
CONFIG_DIR="$HOME/Library/Application Support/$BUNDLE_ID"

echo "========================================"
echo "Config dump for: $BUNDLE_ID"
echo "========================================"
echo ""

if [ ! -d "$CONFIG_DIR" ]; then
    echo "Error: Directory not found: $CONFIG_DIR"
    exit 1
fi

echo "Location: $CONFIG_DIR"
echo ""

# List all files
echo "Files:"
ls -la "$CONFIG_DIR"
echo ""

# Dump each JSON file
for file in "$CONFIG_DIR"/*.json; do
    if [ -f "$file" ]; then
        filename=$(basename "$file")
        echo "----------------------------------------"
        echo "File: $filename"
        echo "----------------------------------------"
        if command -v python3 &> /dev/null; then
            cat "$file" | python3 -m json.tool 2>/dev/null || cat "$file"
        else
            cat "$file"
        fi
        echo ""
    fi
done

echo "========================================"
