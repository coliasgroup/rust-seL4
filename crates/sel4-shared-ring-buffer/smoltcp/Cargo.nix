#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, localCrates, versions, smoltcpWith }:

mk {
  package.name = "sel4-shared-ring-buffer-smoltcp";
  dependencies = {
    inherit (versions) log;
    smoltcp = smoltcpWith [];
    inherit (localCrates)
      sel4-shared-ring-buffer
      sel4-shared-ring-buffer-bookkeeping
      sel4-bounce-buffer-allocator
      sel4-sync-abstractions
    ;
    sel4-externally-shared = localCrates.sel4-externally-shared // { features = [ "unstable" ]; };
  };
  
}
