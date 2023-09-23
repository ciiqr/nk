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

Write-Output '==> identifying os/arch'

$os = 'windows'
$arch = identify_arch

Write-Output '==> download nk'

# paths
$bin_directory = "${HOME}\.nk\bin"
$nk_path = "${bin_directory}\nk.exe"

# create bin directory
New-Item -ItemType Directory -Force -Path $bin_directory

# determine nk url
$nk_url = if ($version -ceq 'latest') {
    "https://github.com/ciiqr/nk/releases/latest/download/nk-${os}-${arch}.exe"
}
else {
    "https://github.com/ciiqr/nk/releases/download/${version}/nk-${os}-${arch}.exe"
}

# download nk binary
(New-Object System.Net.WebClient).Downloadfile($nk_url, $nk_path)

# prepend to path
prepend_to_path -new_path $bin_directory
