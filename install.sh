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

nk::install::identify_jq_arch() {
    declare raw_arch
    raw_arch="$(uname -m)"

    case "$raw_arch" in
        x86_64)
            echo 'amd64'
            ;;
        *)
            echo "$raw_arch"
            ;;
    esac
}

nk::install::usage() {
    echo 'usage: nk-install [--version <version>]'
    echo ''
    echo 'options:'
    echo '  --version <version> version tag ie. "v0.16.0"'
}

nk::install::parse_cli_args() {
    while [[ "$#" -gt 0 ]]; do
        case "$1" in
            --version)
                version="$2"
                shift
                ;;
            -h | --help)
                nk::install::usage
                exit 0
                ;;
            *)
                echo "nk-install: unrecognized option $1" 1>&2
                nk::install::usage 1>&2
                return 1
                ;;
        esac
        shift
    done
}

declare version='latest'

nk::install::parse_cli_args "$@"

echo '==> identifying os/arch'

# determine os
declare os
os="$(nk::install::identify_os)"

# determine arch
declare arch
arch="$(nk::install::identify_arch)"

echo '==> download nk'

# paths
declare bin_directory="${HOME}/.nk/bin"
declare nk_path="${bin_directory}/nk"
declare jq_path="${bin_directory}/jq"

# create bin directory
mkdir -p "$bin_directory"

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
declare jq_arch
jq_arch="$(nk::install::identify_jq_arch)"
declare jq_url="https://github.com/jqlang/jq/releases/download/jq-1.7/jq-${os}-${jq_arch}"

# download jq binary
curl -fsSL "$jq_url" -o "$jq_path"

# make jq executable
chmod +x "$jq_path"

# create completions
echo '==> create completions'
"$nk_path" completion install
