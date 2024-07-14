#
# Copyright 2024, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, versions, localCrates }:

mk {
  package.name = "spawn-task";
  dependencies = {
    object = { version = versions.object; default-features = false; features = [ "read" ]; };
    inherit (localCrates)
      sel4
      sel4-root-task
      sel4-supervising
    ;
  };
}
