# SPDX-FileCopyrightText: 2024 Harish Rajagopal <harish.rajagopals@gmail.com>
#
# SPDX-License-Identifier: GPL-3.0-or-later
{
  callPackage,
  mkShell,
  vscode-with-extensions,
  vscode-extensions,
}:
mkShell {
  inputsFrom = [(callPackage ./rust.nix {})];
  buildInputs = [
    (
      vscode-with-extensions.override {
        vscodeExtensions = with vscode-extensions; [
          rust-lang.rust-analyzer
          tamasfe.even-better-toml
          bbenoist.nix

          vscodevim.vim # you can disable this in extension settings if you want
        ];
      }
    )
  ];
}
