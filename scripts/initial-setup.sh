#!/usr/bin/env bash

set -e

if ! type 'rustup' > /dev/null 2>&1; then
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s - -y --no-modify-path
fi

echo '==> configure git hooks'
git config --local core.hookspath .hooks
