# SPDX-FileCopyrightText: 2024 Harish Rajagopal <harish.rajagopals@gmail.com>
#
# SPDX-License-Identifier: GPL-3.0-or-later
{
  callPackage,
  mkShell,
  rust-analyzer,
  rustfmt,
  clippy,
  pre-commit,
  greetd,
}:
mkShell {
  inputsFrom = [(callPackage ../packages/regreet.nix {})];
  buildInputs = [
    rust-analyzer
    rustfmt
    clippy

    pre-commit

    greetd.greetd # fakegreet
  ];

  shellHook = ''
    echo "Installing pre commit hooks";
    pre-commit install;
  '';
}
