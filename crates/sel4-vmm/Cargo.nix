#
# Copyright 2024, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, versions, localCrates }:

mk {
  package.name = "sel4-vmm";
  dependencies = {
    inherit (localCrates)
      sel4-vmm-interfaces
    ;
  };
}
