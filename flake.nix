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
    pkgsFor = system:
      import nixpkgs {
        config.allowUnfree = true;
        inherit system;
      };
  in {
    formatter = forAllSystems (system: (pkgsFor system).alejandra);

    packages = forAllSystems (system: let
      pkgs = pkgsFor system;
    in
      {
        default = self.packages.${system}.regreet;
      }
      // (pkgs.callPackage ./nix/packages {}));

    devShells = forAllSystems (system: let
      pkgs = pkgsFor system;
    in
      {
        default = self.devShells.${system}.rust;
      }
      // (pkgs.callPackage ./nix/shells {}));
  };
}
