#!/usr/bin/env bash

set -e

# TODO: have parameters for which release of nk to install (default to latest)

echo 'there are no releases setup just yet, so this script does not actually work'

# TODO: determine os: uname -s
declare os='macos' # OR Darwin
# TODO: determine arch: uname -m
declare arch='aarch64' # OR arm64?

declare bin_directory="${HOME}/.nk/bin"
declare nk_path="${bin_directory}/nk"
# declare jq_path="${bin_directory}/jq"

# create bin directory
mkdir -p "$bin_directory"

# download nk binary
curl -fsSL "https://github.com/ciiqr/nk/releases/latest/download/nk-${os}-${arch}" -o "$nk_path"

# make nk executable
chmod +x "$nk_path"

# TODO: download jq binary
# TODO: consider moving this into nk itself (or making helper stuff like this a plugin variant)
# curl -fsSL "https://github.com/ciiqr/nk/releases/latest/download/nk-${os}-${arch}" -o "$nk_path"
# TODO: download official binary if available, else we'll need to provide our own build for macos-arm64: https://gist.github.com/magnetikonline/58eb344e724d878345adc8622f72be13

# make jq executable
# chmod +x "$jq_path"
