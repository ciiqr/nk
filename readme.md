# nk

configuration management for developers

## install

-   Install nk

    ```bash
    curl -fsSL https://raw.githubusercontent.com/ciiqr/nk/HEAD/install.sh | bash
    ```

-   Add to path

    _append to your `~/.zprofile` or `~/.bash_profile`/`~/.bashrc` to make this permanent_

    ```bash
    export PATH="${HOME}/.nk/bin:${PATH}"
    ```

## install (windows)

-   Install nk (via powershell)

    ```powershell
    Set-ExecutionPolicy Bypass -Scope Process -Force
    Invoke-Expression ((New-Object System.Net.WebClient).DownloadString(
        'https://raw.githubusercontent.com/ciiqr/nk/HEAD/install.ps1'
    ))
    ```

-   Add to path

    _for current session only, install script already updated user path which will be used for new sessions_

    ```powershell
    $env:Path = "${HOME}/.nk/bin" + [IO.Path]::PathSeparator + $env:Path
    ```

## setup

-   Create nk config `./.nk.yml`

    <!-- prettier-ignore -->
    ```yaml
    sources:
      - .
    plugins:
      - ciiqr/nk-plugins
    ```

-   Create state config ie. `config.yml` (any `*.yml` except dotfiles `.*.yml`)

    <!-- prettier-ignore -->
    ```yaml
    when: os == "macos"

    packages:
      - homebrew/cask/google-chrome
    ```

-   Provision state

    ```bash
    nk provision
    ```

## local development

-   install dependencies and configure hooks

```bash
./scripts/initial-setup.sh
```

## resources

-   plugins: [ciiqr/nk-plugins](https://github.com/ciiqr/nk-plugins)
-   example config: [ciiqr/dotfiles](https://github.com/ciiqr/dotfiles)
