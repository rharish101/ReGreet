# SPDX-FileCopyrightText: 2024 max_ishere <47008271+max-ishere@users.noreply.github.com>
#
# SPDX-License-Identifier: MIT
{
  description = "Dev tooling for ReGreet";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils?ref=main";
    cargo-fakegreet = {
      url = "github:max-ishere/cargo-fakegreet?ref=main";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
    cargo-fakegreet,
  }:
    flake-utils.lib.eachDefaultSystem (system: let
      pkgs = nixpkgs.legacyPackages.${system};
    in {
      formatter = pkgs.alejandra;

      packages.default = self.packages.${system}.regreet;
      packages.regreet = pkgs.callPackage ./nix/packages/regreet.nix {};

      devShells.default = import ./nix/shells/regreet.nix {
        inherit pkgs;
        inherit (self.packages.${system}) regreet;
        inherit (cargo-fakegreet.packages.${system}) cargo-fakegreet;
      };
    });
}
