#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: MIT
#

{ mk, versions }:

mk {
  package.name = "sel4-sync-abstractions";
  dependencies = {
    inherit (versions) lock_api;
  };
  features = {
    alloc = [];
  };
}
