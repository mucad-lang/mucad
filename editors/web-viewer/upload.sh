#!/usr/bin/env bash
# Copyright © 2026 The µcad authors <info@mucad.org>
# SPDX-License-Identifier: AGPL-3.0-or-later

# Build the web viewer and upload it to the web server of the settings in `.env`.
#
# The build output `dist/` contains all files of the site, e.g. the assets of `src/assets`,
# so it is mirrored to the server: files on the server, which are not part of the site,
# are deleted.
#
# Usage: ./upload.sh [--dry-run]
#   --dry-run  Build, but only show what would be uploaded and deleted.
#
# Requirements: npm, wasm-pack (for `npm run wasm`) and lftp.

set -euo pipefail

DRY_RUN=false
for arg in "$@"; do
    case "$arg" in
        --dry-run) DRY_RUN=true ;;
        -h | --help)
            sed -n '5,14p' "$0" | sed 's/^# \{0,1\}//'
            exit 0
            ;;
        *)
            echo "Unknown argument: $arg (use --dry-run or --help)" >&2
            exit 1
            ;;
    esac
done

# Run in the directory of the web viewer, wherever the script is called from.
cd "$(dirname "$0")"

fail() {
    echo "❌ $*" >&2
    exit 1
}

# --- Settings ---------------------------------------------------------------------------------

[ -f .env ] || fail ".env not found, copy env.example to .env and fill in your values"

# Read the settings literally, without shell expansion; surrounding quotes are removed.
while IFS= read -r line || [ -n "$line" ]; do
    line="${line%$'\r'}"
    # Skip empty lines and comments.
    [[ "$line" =~ ^[[:space:]]*(#|$) ]] && continue
    [[ "$line" == *=* ]] || fail ".env: no setting: $line"
    key="${line%%=*}"
    key="${key//[[:space:]]/}"
    value="${line#*=}"
    if [[ "$value" =~ ^\"(.*)\"$ || "$value" =~ ^\'(.*)\'$ ]]; then
        value="${BASH_REMATCH[1]}"
    fi
    case "$key" in
        UPLOAD_PROTOCOL | UPLOAD_HOST | UPLOAD_PORT | UPLOAD_USER | UPLOAD_PASSWORD | UPLOAD_REMOTE_DIR)
            printf -v "$key" '%s' "$value"
            ;;
        *) fail ".env: unknown setting: $key" ;;
    esac
done <.env

UPLOAD_PROTOCOL="${UPLOAD_PROTOCOL:-sftp}"
UPLOAD_HOST="${UPLOAD_HOST:-}"
UPLOAD_USER="${UPLOAD_USER:-}"
UPLOAD_PASSWORD="${UPLOAD_PASSWORD:-}"
UPLOAD_REMOTE_DIR="${UPLOAD_REMOTE_DIR:-}"

case "$UPLOAD_PROTOCOL" in
    sftp) DEFAULT_PORT=22 ;;
    ftps | ftp) DEFAULT_PORT=21 ;;
    *) fail ".env: UPLOAD_PROTOCOL must be sftp, ftps or ftp, not '$UPLOAD_PROTOCOL'" ;;
esac
UPLOAD_PORT="${UPLOAD_PORT:-$DEFAULT_PORT}"

[ -n "$UPLOAD_HOST" ] || fail ".env: UPLOAD_HOST is missing"
[ -n "$UPLOAD_USER" ] || fail ".env: UPLOAD_USER is missing"
[ -n "$UPLOAD_PASSWORD" ] || fail ".env: UPLOAD_PASSWORD is missing"
# Mirroring deletes files, so the server's root directory is refused.
case "$UPLOAD_REMOTE_DIR" in
    "" | "/" | ".") fail ".env: UPLOAD_REMOTE_DIR must be the directory of the site, not '$UPLOAD_REMOTE_DIR'" ;;
esac

for tool in npm lftp; do
    command -v "$tool" >/dev/null || fail "'$tool' is not installed"
done

# --- Build ------------------------------------------------------------------------------------

npm audit fix --force

echo "⚙️  Building the WebAssembly package..."
npm run wasm
# After the WebAssembly package, which is a dependency in package.json.
echo "📦 Installing dependencies..."
npm install
echo "🏗️  Building the web viewer..."
npm run build

[ -f dist/index.html ] || fail "The build has no dist/index.html"

# --- Upload -----------------------------------------------------------------------------------

# Quote a value for an lftp command.
quote() {
    local value="${1//\\/\\\\}"
    printf '"%s"' "${value//\"/\\\"}"
}

case "$UPLOAD_PROTOCOL" in
    # The host key of an unknown server is accepted on the first connection.
    sftp) SETTINGS="set sftp:auto-confirm yes;" URL="sftp://$UPLOAD_HOST" ;;
    ftps) SETTINGS="set ftp:ssl-force true; set ftp:ssl-protect-data true;" URL="ftp://$UPLOAD_HOST" ;;
    ftp)
        echo "⚠️  Uploading unencrypted with ftp, the password is sent in clear text"
        SETTINGS="set ftp:ssl-allow false;" URL="ftp://$UPLOAD_HOST"
        ;;
esac

MIRROR="mirror --reverse --delete --verbose --parallel=4"
if [ "$DRY_RUN" = true ]; then
    MIRROR="$MIRROR --dry-run"
    echo "🔍 Dry run: showing what would be uploaded to $UPLOAD_PROTOCOL://$UPLOAD_HOST$UPLOAD_REMOTE_DIR"
else
    echo "🚀 Uploading to $UPLOAD_PROTOCOL://$UPLOAD_HOST$UPLOAD_REMOTE_DIR..."
fi

# The password is passed in the environment, so it does not appear in the process list.
LFTP_PASSWORD="$UPLOAD_PASSWORD" lftp -c "$SETTINGS
    open --env-password -u $(quote "$UPLOAD_USER") -p $UPLOAD_PORT $(quote "$URL");
    $MIRROR dist $(quote "$UPLOAD_REMOTE_DIR")"

if [ "$DRY_RUN" = true ]; then
    echo "✅ Dry run finished, nothing was uploaded"
else
    echo "✅ Web viewer uploaded"
fi
