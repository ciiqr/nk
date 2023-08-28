#!/usr/bin/env bash

set -e

echo '==> install brew'
if ! type 'brew' > /dev/null 2>&1; then
    /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
fi

echo '==> install dependencies'
brew install \
    gh \
    jq

# TODO: is there a better alternative available in brew?
sudo curl --output /usr/local/bin/semver \
  https://raw.githubusercontent.com/fsaintjacques/semver-tool/master/src/semver
sudo chmod +x /usr/local/bin/semver

if ! type 'rustup' > /dev/null 2>&1; then
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s - -y --no-modify-path
fi

echo '==> configure git hooks'
git config --local core.hookspath .hooks
