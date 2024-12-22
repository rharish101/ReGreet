<!--
SPDX-FileCopyrightText: 2024 Harish Rajagopal <harish.rajagopals@gmail.com>

SPDX-License-Identifier: GPL-3.0-or-later
-->

# Contributing

## Pre commit

[pre-commit](https://pre-commit.com/) is used for managing hooks that run before each commit (such as clippy), to ensure
code quality. Thus, this needs to be set up only when one intends to commit changes to git.

Firstly, [install pre-commit](https://pre-commit.com/#installation) itself. Next, install pre-commit hooks:
```sh
pre-commit install
```

Now, pre-commit should ensure that the code passes all linters locally before committing. This will save time when
creating PRs, since these linters also run in CI, and thus fail code that hasn't been linted well.

## Nix dev shells

Simply run `nix develop .#vscode --command code .` to get VS Code set up with all the extensions you need. The default
shell doesn't have any text editors configred.

You can also use nix without flakes if you choose to. See the [nix documentation in this repo](nix/README.md) for more
details.

## Testing

You can run ReGreet without a greetd socket using `--demo` flag. It also disables some of the features such as logging
to a file.

```sh
regreet --demo
```

Since the demo mode doesn't use greetd, authentication is done using hardcoded credentials within the codebase. These
credentials are logged with the warning log level, so that you don't have to read the source code.

-----

Alternatively you can use `fakegreet` to emulate a running greetd daemon. Please keep in mind that it's behavior doesn't
match the real thing. 

```sh
fakegreet 'cargo run'
```

Fakegreet credentials (taken from source code):

||Value|
|---|---|
|User|user|
|Password|password|
|7+2|9|