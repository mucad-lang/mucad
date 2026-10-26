#!/bin/bash

set -euo pipefail

# Run in the root dir of this repository.
cd "$(dirname "$0")/.."

cargo update
cargo clippy

cargo xtask update-copyright


echo "Pre-publish script run successfully. Do not forget to run 'git tag' before publish!"
