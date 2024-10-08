# SPDX-FileCopyrightText: 2024 Harish Rajagopal <harish.rajagopals@gmail.com>
#
# SPDX-License-Identifier: GPL-3.0-or-later
{
  description = "Dev tooling for ReGreet";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
  };

  outputs = {
    self,
    nixpkgs,
  }: let
    supportedSystems = ["x86_64-linux"];
    forAllSystems = nixpkgs.lib.genAttrs supportedSystems;
    pkgsFor = nixpkgs.legacyPackages;
  in {
    formatter = forAllSystems (system: pkgsFor.${system}.alejandra);

    packages = forAllSystems (system: let
      pkgs = pkgsFor.${system};
    in rec {
      default = regreet;
      regreet = pkgs.callPackage ./default.nix {inherit pkgs;};
    });

    devShells = forAllSystems (system: let
      pkgs = pkgsFor.${system};
    in {
      default = pkgs.callPackage ./shell.nix {inherit pkgs;};

      vscode = pkgs.callPackage ./nix/vscode-shell.nix {
        pkgs = import nixpkgs {
          inherit system;
          config.allowUnfree = true;
        };
      };
    });
  };
}
