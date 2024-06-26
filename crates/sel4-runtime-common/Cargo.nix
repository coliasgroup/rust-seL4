#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, localCrates, versions, unwindingWith }:

mk {
  package.name = "sel4-runtime-common";
  dependencies = {
    inherit (versions) cfg-if;
    inherit (localCrates) sel4-panicking-env sel4-elf-header;
    sel4 = localCrates.sel4 // { default-features = false; optional = true; };
    sel4-initialize-tls = localCrates.sel4-initialize-tls // { features = [ "on-stack" ]; optional = true; };
  };
  target."cfg(not(target_arch = \"arm\"))".dependencies = {
    unwinding = unwindingWith [] // { optional = true; };
  };
  features = {
    tls = [ "dep:sel4-initialize-tls" "dep:sel4" ];
    start = [];
    std = [];
  };
}
