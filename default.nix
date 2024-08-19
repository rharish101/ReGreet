# SPDX-FileCopyrightText: 2024 Harish Rajagopal <harish.rajagopals@gmail.com>
#
# SPDX-License-Identifier: GPL-3.0-or-later
{pkgs ? import <nixpkgs> {}}: let
  manifest = (pkgs.lib.importTOML ./Cargo.toml).package;
in
  pkgs.rustPlatform.buildRustPackage rec {
    pname = manifest.name;
    inherit (manifest) version;
    cargoLock.lockFile = ./Cargo.lock;
    src = pkgs.lib.cleanSource ./.;

    buildFeatures = ["gtk4_8"];

    nativeBuildInputs = with pkgs; [pkg-config wrapGAppsHook4];
    buildInputs = with pkgs; [glib gtk4 pango librsvg];
  }
