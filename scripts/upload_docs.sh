#!/bin/bash
set -e

# Check for flags
DRY_RUN=""
for arg in "$@"; do
    case "$arg" in
        --dry-run|-d)
            DRY_RUN="--dry-run"
            echo "DRY-RUN MODE - No files will be uploaded"
            echo ""
            ;;
    esac
done

# Get script directory and project root
SCRIPT_DIR=$(dirname "$0")
PROJECT_ROOT=$(cd "$SCRIPT_DIR/.." && pwd)
BOOKS_DIR="$PROJECT_ROOT/books"

# Load .env file
if [ -f "$SCRIPT_DIR/.env" ]; then
    export $(grep -v '^#' "$SCRIPT_DIR/.env" | xargs)
else
    echo "Error: .env file not found!"
    exit 1
fi

# --- Configuration (Ensure these match your script) ---
DEST_DIR="${DEST_DIR:-./target/docs}" # The local folder containing /v1.0.0/ etc.
ZIP_FILE="docs.zip"
FTP_REMOTE_PATH="${FTP_REMOTE_PATH:-/docs.mucad.org/httpdocs}"

# Safety validation
if [ ! -d "$DEST_DIR" ]; then
    echo "❌ ERROR: Local destination directory '$DEST_DIR' does not exist." >&2
    exit 1
fi
# Secure Environment Variables Check
if [ -z "$FTP_PW" ]; then
    echo "❌ ERROR: \$FTP_PW environment variable is not set." >&2
    exit 1
fi

# Ensure sshpass is installed locally before proceeding
if ! command -v sshpass &> /dev/null; then
    echo "❌ ERROR: 'sshpass' utility is not installed on this system." >&2
    exit 1
fi

if [ ! -d "$DEST_DIR" ]; then
    echo "❌ ERROR: Local destination directory '$DEST_DIR' does not exist." >&2
    exit 1
fi

# 1. Archive the input directory locally
echo "📦 Packing target directory into ${ZIP_FILE}..."
(cd "$DEST_DIR" && zip -r "$ZIP_FILE" .)

# 2. Build the flat, newline-free lftp command (avoids "unmatched dquote" errors)
echo "⚙️  Generating remote deployment sequence..."
if [ -n "$DRY_RUN" ]; then
    echo "🔍 [DRY RUN] Simulating upload payload..."
    LFTP_COMMANDS="set ftp:ssl-force true; echo 'Simulating file transfer...'; quit"
else
    # lftp puts the file, then exits. We handle unzip separately via sshpass.
    LFTP_COMMANDS="set ftp:ssl-force true; put -O $FTP_REMOTE_PATH "$DEST_DIR/$ZIP_FILE"; quit"
fi

# 4. Step A: Upload the Archive using lftp's native inline authentication
echo "🚀 Transporting binary package payload..."
lftp -e "$LFTP_COMMANDS" sftp://"$FTP_USER":"$FTP_PW"@"$FTP_HOST":"$FTP_PORT"

# 5. Step B: Remote extraction via sshpass + SSH
if [ -z "$DRY_RUN" ]; then
    echo "🔓 Extracting archive remotely using sshpass..."

    # sshpass -e flags instructs it to read the password directly from $SSHPASS.
    # We pass our $FTP_PW value directly into it for this step.
    export SSHPASS="$FTP_PW"

    sshpass -e ssh -o StrictHostKeyChecking=no -p "$FTP_PORT" "$FTP_USER@$FTP_HOST" \
        "cd $FTP_REMOTE_PATH && unzip -o $ZIP_FILE && rm $ZIP_FILE"

    # Unset immediately after execution to keep environment pristine
    unset SSHPASS
fi

# 6. Local Housekeeping Cleanup
rm -f "$DEST_DIR/$ZIP_FILE"
echo -e "\n🎉 Documentation architecture uploaded and extracted successfully!"
