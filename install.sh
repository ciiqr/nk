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

echo '==> identifying os/arch'

# determine os
declare os
os="$(nk::install::identify_os)"

# determine arch
declare arch
arch="$(nk::install::identify_arch)"

# paths
declare bin_directory="${HOME}/.nk/bin"
declare nk_path="${bin_directory}/nk"
declare jq_path="${bin_directory}/jq"

# create bin directory
mkdir -p "$bin_directory"

echo '==> download nk'

# determine nk url
declare nk_url
if [[ "$version" == 'latest' ]]; then
    nk_url="https://github.com/ciiqr/nk/releases/latest/download/nk-${os}-${arch}"
else
    nk_url="https://github.com/ciiqr/nk/releases/download/${version}/nk-${os}-${arch}"
fi

# download nk binary
curl -fsSL "$nk_url" -o "$nk_path"

# make nk executable
chmod +x "$nk_path"

echo '==> download jq'

# derermine jq url
declare jq_url
if [[ "$os" == 'macos' && "$arch" == 'aarch64' ]]; then
    jq_url='https://github.com/ciiqr/jq-macos-arm/releases/latest/download/jq'
else
    declare jq_binary
    if [[ "$os" == 'linux' && "$arch" == 'x86_64' ]]; then
        jq_binary='jq-linux64'
    elif [[ "$os" == 'macos' && "$arch" == 'x86_64' ]]; then
        jq_binary='jq-osx-amd64'
    else
        echo "unrecognized os/arch: ${os}/${arch}"
        exit 1
    fi
    jq_url="https://github.com/stedolan/jq/releases/download/jq-1.6/${jq_binary}"
fi

# download jq binary
curl -fsSL "$jq_url" -o "$jq_path"

# make jq executable
chmod +x "$jq_path"
