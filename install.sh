#!/usr/bin/env bash

set -e

nk::install::identify_os() {
    case "$OSTYPE" in
        darwin*)
            echo 'macos'
            ;;
        linux*)
            echo 'linux'
            ;;
        *)
            echo "unrecognized os: ${OSTYPE}"
            return 1
            ;;
    esac
}

nk::install::identify_arch() {
    declare raw_arch
    raw_arch="$(uname -m)"

    case "$raw_arch" in
        arm64)
            echo 'aarch64'
            ;;
        *)
            echo "$raw_arch"
            ;;
    esac
}

# TODO: need optional parameter: --version 'v0.4.0' (default to latest)
declare version='latest'

# determine os
declare os
os="$(nk::install::identify_os)"

# determine arch
declare arch
arch="$(nk::install::identify_arch)"

declare bin_directory="${HOME}/.nk/bin"
declare nk_path="${bin_directory}/nk"
# declare jq_path="${bin_directory}/jq"

# create bin directory
mkdir -p "$bin_directory"

# determine download url
declare nk_url
if [[ "$version" == 'latest' ]]; then
    nk_url="https://github.com/ciiqr/nk/releases/latest/download/nk-${os}-${arch}"
else
    nk_url="https://github.com/ciiqr/nk/releases/download/${version}/nk-${os}-${arch}"
fi

# download nk binary
echo curl -fsSL "$nk_url" -o "$nk_path"

# make nk executable
chmod +x "$nk_path"

# TODO: download jq binary
# TODO: consider moving this into nk itself (or making helper stuff like this a plugin variant)
# curl -fsSL "https://github.com/ciiqr/nk/releases/latest/download/nk-${os}-${arch}" -o "$nk_path"
# TODO: download official binary if available, else we'll need to provide our own build for macos-arm64: https://gist.github.com/magnetikonline/58eb344e724d878345adc8622f72be13

# make jq executable
# chmod +x "$jq_path"
