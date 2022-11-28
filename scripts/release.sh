#!/usr/bin/env bash

set -e

# make sure running on arm/macos
if [[ "$OSTYPE" != darwin* || "$(uname -m)" != 'arm64' ]]; then
    echo 'release: must be run from an apple silicon mac'
    exit 1
fi

# create temp dir
declare temp_dir
temp_dir="$(mktemp -d)"
on_exit() {
    rm -r "$temp_dir"
}
trap on_exit EXIT

# create assets dir
declare assets_dir="${temp_dir}/assets"
mkdir "$assets_dir"

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
cargo generate-lockfile

# commit & tag
echo '==> commit & tag'
git add Cargo.toml Cargo.lock
git commit -m "bumped version to ${version}"
declare tag="v${version}"
git tag "$tag"
git push
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
    conclusion="$(jq -r '.conclusion' <<<"$output")"
    runId="$(jq -r '.databaseId' <<<"$output")"

    sleep 1
done

if [[ "$conclusion" != 'success' ]]; then
    echo "remote builds failed: ${output}"
    exit 1
fi

# download remote builds
echo '==> download build artifacts'
gh run download "$runId" --dir "$temp_dir"

# move binaries to assets directory
mv './target/release/nk' "${assets_dir}/nk-macos-aarch64"
mv "${temp_dir}/nk-macos-x86_64/nk" "${assets_dir}/nk-macos-x86_64"
mv "${temp_dir}/nk-linux-x86_64/nk" "${assets_dir}/nk-linux-x86_64"
# TODO: re-enable windows releases...
# mv "${temp_dir}/nk-windows-x86_64.exe/nk.exe" "${assets_dir}/nk-windows-x86_64.exe"

# create release
echo '==> create release'
gh release create \
    --title "$tag" \
    --notes '' \
    "$tag" \
    "${assets_dir}/"*
