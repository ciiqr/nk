#!/usr/bin/env bash

set -e

# make sure running on arm/macos
if [[ "$OSTYPE" != darwin* || "$(uname -m)" != 'arm64' ]]; then
    echo 'release: must be run from an apple silicon mac'
    exit 1
fi

# make sure git tags are up to date
git fetch --tags

# get existing version
declare existing_version
existing_version="$(git tag -l --sort=-v:refname | head -1)"
if [[ -z "$existing_version" ]]; then
    existing_version='0.0.0'
fi

# bump to new version
declare version
version="$(semver bump minor "$existing_version")"

# update version in code
cargo set-version "$version"

# commit & tag
git add Cargo.toml Cargo.lock
git commit -m "bumped version to ${version}"
declare tag="v${version}"
git tag "$tag"
git push origin "$tag"

# build macos arm
cargo build --release

# TODO: wait for remote windows/linux builds
declare output
declare conclusion
declare runId
while [[ -n "$conclusion" ]]; do
    output="$(gh run list -w .github/workflows/build.yml --branch "$tag" --json 'status,conclusion,databaseId' --jq '.[] | select(.status == "completed")' | head -1)"
    conclusion="$(jq '.conclusion' <<< "$output")"
    runId="$(jq '.databaseId' <<< "$output")"
done

# download remote builds
# TODO: maybe download to a temp dir instead?
gh run download "$runId" --dir './target/release'

# create release
gh release create \
    --title "$tag" \
    "$tag" \
    './target/release/nk#nk-macos-aarch64' \
    './target/release/nk-linux-x86_64#nk-linux-x86_64' \
    './target/release/nk-windows-x86_64.exe/nk.exe#nk-windows-x86_64.exe'
