#
# Copyright 2024, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, versions, localCrates }:

mk {
  package.name = "sel4-supervising";
  dependencies = {
    inherit (versions) zerocopy;
    inherit (localCrates) sel4;
  };
}
