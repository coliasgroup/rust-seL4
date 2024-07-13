#
# Copyright 2024, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, localCrates, versions }:

mk {
  package.name = "sel4-vmm-payload-cli";
  dependencies = {
    inherit (versions)
      clap
      anyhow
      num
    ;
    inherit (localCrates)
      sel4-synthetic-elf
    ;
  };
}
