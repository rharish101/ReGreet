# SPDX-FileCopyrightText: 2024 Harish Rajagopal <harish.rajagopals@gmail.com>
#
# SPDX-License-Identifier: GPL-3.0-or-later
{pkgs ? import <nixpkgs> {}}:
pkgs.mkShell {
  inputsFrom = [(pkgs.callPackage ../shell.nix {})];
  buildInputs = with pkgs; [
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
