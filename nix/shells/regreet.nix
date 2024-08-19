# SPDX-FileCopyrightText: 2024 max_ishere <47008271+max-ishere@users.noreply.github.com>
#
# SPDX-License-Identifier: MIT
{
  pkgs,
  regreet,
  cargo-fakegreet,
}:
pkgs.mkShell {
  inputsFrom = [regreet];
  buildInputs = with pkgs; [
    rust-analyzer
    rustfmt
    clippy

    pre-commit

    greetd.greetd # fakegreet
    cargo-fakegreet
  ];

  shellHook = ''
    echo "Installing pre commit hooks";
    pre-commit install;
  '';
}
