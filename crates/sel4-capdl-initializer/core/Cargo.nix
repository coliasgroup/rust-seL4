#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, localCrates, versions }:

mk {
  package.name = "sel4-capdl-initializer-core";
  dependencies = {
    inherit (versions) log;
    inherit (localCrates)
      sel4
      sel4-supervising
    ;
    sel4-capdl-initializer-types = localCrates.sel4-capdl-initializer-types // { features = [ "sel4" ]; };
  };
}
