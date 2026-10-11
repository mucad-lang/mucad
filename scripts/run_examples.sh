#!/bin/bash

# Directory to search (default = current directory)
DIR="./examples"

set -euo pipefail

while IFS= read -r -d '' file; do
    echo "Exporting: $file"
    cargo run --release -- export "$file"
done < <(
    find "$DIR" -type f \( \
        -iname "*.mu" -o \
    \) -print0
)
