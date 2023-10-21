#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, localCrates, versions }:

mk {
  package.name = "sel4-capdl-initializer-core";
  nix.local.dependencies = with localCrates; [
    sel4-capdl-initializer-types
    sel4
  ];
  dependencies = {
    sel4-capdl-initializer-types.features = [ "sel4" ];
    inherit (versions) log;
  };
}
