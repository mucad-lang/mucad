#!/bin/bash

cargo update
cargo clippy

cargo xtask update-copyright


echo "Pre-publish script run successfully. Do not forgot to run 'git tag' before publish!"
