#
# Copyright 2024, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, versions, localCrates }:

mk {
  package.name = "spawn-thread";
  dependencies = {
    inherit (versions) cfg-if;
    inherit (localCrates)
      sel4
      sel4-root-task
      sel4-elf-header
      sel4-stack
      sel4-initialize-tls
      sel4-supervising
    ;
  };
}
