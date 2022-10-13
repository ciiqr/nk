# nk

configuration management for humans

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

    <!-- TODO: need remote plugins to make this at all reasonable without more steps here -->

    ```yaml
    sources:
        - .
    plugins:
        - ../nk-plugins/brew
    ```

-   Create state config ie. `config.yml` (any `*.yml` except dotfiles `.*.yml`)

    ```yaml
    when: os == "macos"

    packages:
        - homebrew/cask/google-chrome
    ```

-   Provision state

    ```bash
    nk provision
    ```
