#!/usr/bin/env bash

set -e

# make sure running on arm/macos
if [[ "$OSTYPE" != darwin* || "$(uname -m)" != 'arm64' ]]; then
    echo 'release: must be run from an apple silicon mac'
    exit 1
fi

# make sure git tags are up to date
echo '==> update tags'
git fetch --tags

# get existing version
echo '==> get existing version'
declare existing_version
existing_version="$(git tag -l --sort=-v:refname | head -1)"
if [[ -z "$existing_version" ]]; then
    existing_version='0.0.0'
fi

# bump to new version
echo '==> bump version'
declare version
version="$(semver bump minor "$existing_version")"

# update version in code
echo '==> set version in code'
cargo set-version "$version"

# commit & tag
echo '==> commit & tag'
git add Cargo.toml Cargo.lock
git commit -m "bumped version to ${version}"
declare tag="v${version}"
git tag "$tag"
git push origin "$tag"

# build macos arm
echo '==> build macos arm'
cargo build --release

# wait for remote windows/linux builds
echo '==> wait for remote builds'
declare output
declare conclusion
declare runId
while [[ -z "$conclusion" ]]; do
    output="$(gh run list -w .github/workflows/build.yml --branch "$tag" --json 'status,conclusion,databaseId' --jq '.[] | select(.status == "completed")' | head -1)"
    conclusion="$(jq '.conclusion' <<< "$output")"
    runId="$(jq '.databaseId' <<< "$output")"

    sleep 1
done

if [[ "$conclusion" != 'success' ]]; then
    echo "remote builds failed: ${output}"
    exit 1
fi

# delete old builds
rm -rf ./target/release/nk-{linux,windows}*

# download remote builds
# TODO: maybe download to a temp dir instead?
echo '==> download build artifacts'
gh run download "3375321544" --dir './target/release'

# create release
echo '==> create release'
gh release create \
    --title "$tag" \
    "$tag" \
    './target/release/nk#nk-macos-aarch64' \
    './target/release/nk-linux-x86_64/nk#nk-linux-x86_64' \
    './target/release/nk-windows-x86_64.exe/nk.exe#nk-windows-x86_64.exe'
