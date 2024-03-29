Param (
    [Parameter(HelpMessage = 'version tag ie. "v0.20.0"')]
    [string]$version = 'latest'
)

# stop on first error
$ErrorActionPreference = "Stop"

function identify_arch() {
    switch ($env:PROCESSOR_ARCHITECTURE) {
        "AMD64" {
            "x86_64"
        }
        # NOTE: not currently supported
        # "ARM64" {
        #     "aarch64"
        # }
        default {
            $env:PROCESSOR_ARCHITECTURE
        }
    }
}

function prepend_to_path() {
    Param (
        [string]$new_path
    )

    $pathString = [Environment]::GetEnvironmentVariable("Path", "User")
    $pathArray = $pathString.Split(';')

    # new path not in existing path
    if ($pathArray -cnotcontains $new_path) {
        [Environment]::SetEnvironmentVariable(
            "Path",
            $new_path +
            [IO.Path]::PathSeparator +
            $pathString,
            "User"
        )
    }
}

function hide_item() {
    Param (
        [string]$path
    )

    $item = Get-Item $path -Force
    if (!($item.Attributes -band "Hidden")) {
        $item.Attributes = $item.Attributes -bor "Hidden"
    }
}

Write-Output '==> identifying os/arch'

$os = 'windows'
$arch = identify_arch

Write-Output '==> download nk'

# paths
$nk_dir = "${HOME}\.nk"
$bin_directory = "${nk_dir}\bin"
$nk_path = "${bin_directory}\nk.exe"

# create bin directory
[void](New-Item -ItemType Directory -Force -Path $bin_directory)

# hide nk dir
hide_item $nk_dir

# determine nk url
$nk_url = if ($version -ceq 'latest') {
    "https://github.com/ciiqr/nk/releases/latest/download/nk-${os}-${arch}.exe"
}
else {
    "https://github.com/ciiqr/nk/releases/download/${version}/nk-${os}-${arch}.exe"
}

# download nk binary
(New-Object System.Net.WebClient).Downloadfile($nk_url, $nk_path)

Write-Output '==> prepend to path'

# prepend to path
prepend_to_path -new_path $bin_directory

# TODO: not sure if there's a standard path for this stuff on windows? might just need to tell the user to add it to their powershell profile...
# Write-Output '==> create powershell completions'
# # create completions
# Write-Output '==> create completions'
# & $nk_path completion install
