# SPDX-FileCopyrightText: 2024 Harish Rajagopal <harish.rajagopals@gmail.com>
#
# SPDX-License-Identifier: GPL-3.0-or-later
{
  lib,
  callPackage,
}: let
  inherit (builtins) map listToAttrs filter attrNames readDir;
  inherit (lib) removeSuffix;

  ls = attrNames (readDir ./.);
  notThisFile = name: name != "default.nix";
  rmDotNix = removeSuffix ".nix";
  mkAttr = file: {
    name = rmDotNix file;
    value = callPackage ./${file} {};
  };
in
  listToAttrs (map mkAttr (filter notThisFile ls))
