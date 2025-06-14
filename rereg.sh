#!/bin/bash

# Re-registration script for DirXpand
# Re-registers the app with macOS Launch Services to update file associations

set -e  # Exit on any error

echo "Re-registering DirXpand with macOS Launch Services..."

# Unregister the app first (ignore errors if not previously registered)
/System/Library/Frameworks/CoreServices.framework/Frameworks/LaunchServices.framework/Support/lsregister -u "$(pwd)/DirXpand.app" 2>/dev/null || true

# Register the app with macOS
/System/Library/Frameworks/CoreServices.framework/Frameworks/LaunchServices.framework/Support/lsregister -f "$(pwd)/DirXpand.app"

echo "✅ Re-registration complete! DirXpand file associations updated."
echo "You can now right-click .dir files and choose 'Open With' → DirXpand"
