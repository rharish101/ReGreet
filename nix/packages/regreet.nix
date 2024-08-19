# SPDX-FileCopyrightText: 2024 max_ishere <47008271+max-ishere@users.noreply.github.com>
#
# SPDX-License-Identifier: MIT
{
  buildFeatures ? ["gtk4_8"],
  lib,
  rustPlatform,
  pkg-config,
  wrapGAppsHook4,
  glib,
  gtk4,
  pango,
  librsvg,
}: let
  manifest = (lib.importTOML ../../Cargo.toml).package;
in
  rustPlatform.buildRustPackage rec {
    pname = manifest.name;
    inherit (manifest) version;
    cargoLock.lockFile = ../../Cargo.lock;
    src = lib.cleanSource ../..;

    inherit buildFeatures;

    nativeBuildInputs = [pkg-config wrapGAppsHook4];
    buildInputs = [glib gtk4 pango librsvg];
  }
