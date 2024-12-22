<!--
SPDX-FileCopyrightText: 2024 Harish Rajagopal <harish.rajagopals@gmail.com>

SPDX-License-Identifier: GPL-3.0-or-later
-->

# Nix

## Structure

|Path||
|---|---|
|[`/nix/packages`](packages)|Filenames are keys in both `flake.packages` and attributes in [`/default.nix`](../default.nix)|
|[`/nix/shells`](shells)|Filenames are keys in both `flake.devShells` and attributes in [`/shell.nix`](../shell.nix)|

## Loading a shell / building packages

<details><summary>With flakes</summary>

See the flake source for what `.#default` points to.

```sh
nix develop .#<filename>
nix build .#<filename>

# Example

nix develop .#vscode
# Loads ./shells/vscode.nix
```

</details>

-----

<details><summary>Without flakes</summary>

You have to select an attribute with `-A`. `default` is not set (it doesnt work like that in `nix-*` commnands)!

```sh
nix-shell -A <filename>
nix-build -A <filename>

# Example

nix-build -A regreet
# Builds ./packages/regreet.nix
```

</details>