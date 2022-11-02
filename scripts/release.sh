#!/usr/bin/env bash

set -e

# TODO: make sure running on m1 darwin
if [[ "$OSTYPE" != darwin* || "$(/usr/bin/uname -m)" != 'arm64' ]]; then
    echo 'release: must be run from an apple silicon mac'
    exit 1
fi


# TODO: determine new version (bump minor)

# TODO: update version in code
# cargo set-version "$VERSION"

# TODO: commit & tag
# git add Cargo.toml Cargo.lock
# git commit -m "bumped version to ${VERSION}"
# git tag "$VERSION"
# git push origin "$VERSION"

# TODO: build macos arm

# TODO: wait for remote windows/linux builds

# TODO: download remote builds

# TODO: create release
# TODO: attach binaries with -a nk-macos-arm64 -a nk-windows etc.
# hub release create -m "$VERSION" "$VERSION"
