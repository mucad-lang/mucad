#!/bin/bash

# Use this script to publish a new µcad version to crates.io
# Note: Execute this script from the root dir of this repository.

command -v jq >/dev/null 2>&1 || { echo "Please install package 'jq'!"; exit 1; }

# Check if we are on a git tag and if it matches the crate version.

ARGS=$@ # All arguments (e.g. `--dry-run`) will be appended to `cargo publish`
GIT_TAG=$(git describe --tags --exact-match 2>/dev/null)
CRATE_VERSION=v`cargo metadata --no-deps --format-version 1 | jq -r '.packages[0].version'`

if [ -z "$GIT_TAG" ]; then
    echo "Warning: You are NOT a on git tag, this means you can only publish in dry run mode!"
    echo "Tag your branch with '$CRATE_VERSION' to publish in real."
    ARGS+=" --dry-run"
else
    echo "You are on git tag: ${GIT_TAG}"

    if [ "$GIT_TAG" != "$CRATE_VERSION" ]; then
        echo "Error: Git tag ($GIT_TAG) does not match Cargo.toml version ($CRATE_VERSION)"
        echo "Create a tag with version ${CRATE_VERSION} first before you publish."
        exit 1
    fi
fi



PACKAGES=(
    "mucad-core"
    "mucad-macros"
    "mucad-lang-base"
    "mucad-lang-parse"
    "mucad-lang"
    "mucad-lang-format"
    "mucad-lang-markdown"
    "mucad-export"
    "mucad-import"
    "mucad-builtin"
    "mucad-std"
    "mucad-docgen"
    "mucad-driver"
    "mucad"
    "mucad-viewer-ipc"
    "mucad-viewer"
    "mucad-lsp"
)

# Publish all packages.
for package in "${PACKAGES[@]}"; do
    cargo publish -p $package $ARGS
done
