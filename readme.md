# nk

config focused configuration management

## setup

-   Install nk

    ```bash
    curl -fsSL https://raw.githubusercontent.com/ciiqr/nk/HEAD/install.sh | bash
    ```

-   Add to path

    _append to your `~/.zprofile` or `~/.bash_profile`/`~/.bashrc` to make this permanent_

    ```bash
    export PATH="${PATH}:${HOME}/.nk/bin"
    ```

-   Create nk config `./.nk.yml`

    <!-- prettier-ignore -->
    ```yaml
    sources:
      - .
    plugins:
      - ciiqr/nk-plugins#brew
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

## resources

-   plugins: [ciiqr/nk-plugins](https://github.com/ciiqr/nk-plugins)
-   example config: [ciiqr/dotfiles](https://github.com/ciiqr/dotfiles)
