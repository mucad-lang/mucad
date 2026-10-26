#!/bin/bash

# Use this script to publish a new µcad version to crates.io.
# All arguments (e.g. `--dry-run`) are appended to `cargo publish`.
#
# All crates of the workspace are published in the order of their dependencies, except crates
# with `publish = false`, e.g. `xtask` and `mucad-wasm`.

set -euo pipefail

# Run in the root dir of this repository.
cd "$(dirname "$0")/.."

command -v jq >/dev/null 2>&1 || { echo "Please install package 'jq'!"; exit 1; }

ARGS=("$@")
GIT_TAG=$(git describe --tags --exact-match 2>/dev/null || true)
CRATE_VERSION=v$(cargo metadata --no-deps --format-version 1 \
    | jq -r '.packages[] | select(.name == "mucad") | .version')

# Check if we are on a git tag and if it matches the crate version.
if [ -z "$GIT_TAG" ]; then
    echo "Warning: You are NOT on a git tag, this means you can only publish in dry run mode!"
    echo "Tag your branch with '$CRATE_VERSION' to publish in real."
    ARGS+=(--dry-run)
else
    echo "You are on git tag: ${GIT_TAG}"

    if [ "$GIT_TAG" != "$CRATE_VERSION" ]; then
        echo "Error: Git tag ($GIT_TAG) does not match Cargo.toml version ($CRATE_VERSION)"
        echo "Create a tag with version ${CRATE_VERSION} first before you publish."
        exit 1
    fi
fi

cargo publish --workspace "${ARGS[@]}"
