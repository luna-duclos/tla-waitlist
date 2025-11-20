#!/bin/bash
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BACKEND_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
MACROS_DIR="$BACKEND_DIR/eve-data-macros"

cd "$BACKEND_DIR"

# Check if cargo is available
if ! command -v cargo &> /dev/null; then
    echo "Error: cargo not found. Please install Rust."
    exit 1
fi

# Check for required tools
if ! command -v wget &> /dev/null && ! command -v curl &> /dev/null; then
    echo "Error: wget or curl not found. Please install one of them."
    exit 1
fi

if ! command -v unzip &> /dev/null && ! command -v tar &> /dev/null; then
    echo "Error: unzip or tar not found. Please install one of them."
    exit 1
fi

# Create temporary directory for SDE
TEMP_DIR=$(mktemp -d)
trap "rm -rf $TEMP_DIR" EXIT

SDE_URL="https://developers.eveonline.com/static-data/eve-online-static-data-latest-jsonl.zip"
SDE_FILE="$TEMP_DIR/sde-latest.jsonl.zip"

if command -v wget &> /dev/null; then
    wget -q --show-progress -O "$SDE_FILE" "$SDE_URL" || {
        echo "Error: Failed to download SDE from $SDE_URL"
        exit 1
    }
elif command -v curl &> /dev/null; then
    curl -L --progress-bar -o "$SDE_FILE" "$SDE_URL" || {
        echo "Error: Failed to download SDE from $SDE_URL"
        exit 1
    }
else
    echo "Error: Neither wget nor curl found. Cannot download SDE."
    exit 1
fi

if [ ! -f "$SDE_FILE" ]; then
    echo "Error: Download failed - file not found"
    exit 1
fi

if command -v unzip &> /dev/null; then
    unzip -q "$SDE_FILE" -d "$TEMP_DIR" || {
        echo "Error: Failed to extract SDE archive"
        exit 1
    }
elif command -v tar &> /dev/null; then
    # Try tar as fallback (though SDE is typically zip)
    tar -xzf "$SDE_FILE" -C "$TEMP_DIR" || {
        echo "Error: Failed to extract SDE archive. Please install unzip."
        exit 1
    }
else
    echo "Error: unzip not found. Please install unzip to extract the SDE."
    exit 1
fi

# Find the extracted SDE directory
# The archive might extract to a subdirectory or directly to TEMP_DIR
SDE_ROOT=$(find "$TEMP_DIR" -type d -name "eve-online-static-data-*-jsonl" | head -1)
if [ -z "$SDE_ROOT" ]; then
    # Fallback: look for any directory containing JSONL files
    FIRST_JSONL=$(find "$TEMP_DIR" -name "*.jsonl" -type f | head -1)
    if [ -z "$FIRST_JSONL" ]; then
        echo "Error: Could not find any JSONL files in extracted archive"
        echo "Archive structure:"
        find "$TEMP_DIR" -type d -maxdepth 2 | head -10
        echo ""
        echo "Files in temp directory:"
        ls -la "$TEMP_DIR" | head -10
        exit 1
    fi
    SDE_ROOT=$(dirname "$FIRST_JSONL")
    # If files are directly in TEMP_DIR, that's fine - use TEMP_DIR as SDE_ROOT
    if [ "$SDE_ROOT" = "$TEMP_DIR" ]; then
        SDE_ROOT="$TEMP_DIR"
    fi
fi

# Check for required files
if [ ! -f "$SDE_ROOT/types.jsonl" ]; then
    echo "Error: types.jsonl not found in $SDE_ROOT/"
    echo "Looking for type files..."
    find "$SDE_ROOT" -name "*type*.jsonl" | head -5
    echo "Available files:"
    ls -la "$SDE_ROOT/" | head -10
    exit 1
fi

if [ ! -f "$SDE_ROOT/groups.jsonl" ]; then
    echo "Error: groups.jsonl not found in $SDE_ROOT/"
    exit 1
fi

if [ ! -f "$SDE_ROOT/typeDogma.jsonl" ]; then
    echo "Error: typeDogma.jsonl not found in $SDE_ROOT/"
    exit 1
fi

cd "$BACKEND_DIR"
if ! cargo build --bin convert_sde_to_sqlite -p eve_data_macros --release 2>&1 | grep -E "(Compiling|Finished|error)"; then
    # If grep finds nothing, that's okay - just check if build succeeded
    true
fi

# Check if build actually succeeded (workspace target is at backend root)
if [ ! -f "$BACKEND_DIR/target/release/convert_sde_to_sqlite" ]; then
    echo "Error: Failed to build converter binary"
    echo "Checking for binary in different locations..."
    find "$BACKEND_DIR/target" -name "convert_sde_to_sqlite" -type f 2>/dev/null
    exit 1
fi

# Pass SDE_ROOT directly - files are in the root, not in fsd/ subdirectory
"$BACKEND_DIR/target/release/convert_sde_to_sqlite" "$SDE_ROOT"

if [ ! -f "sqlite-shrunk.sqlite" ]; then
    echo "Error: Conversion failed - sqlite-shrunk.sqlite not created"
    exit 1
fi

# Check database size
DB_SIZE=$(du -h sqlite-shrunk.sqlite | cut -f1)
echo "Conversion complete: sqlite-shrunk.sqlite ($DB_SIZE)"

