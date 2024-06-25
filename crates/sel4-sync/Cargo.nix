#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: MIT
#

{ mk, mkDefaultFrontmatterWithReuseArgs, defaultReuseFrontmatterArgs, versions, localCrates }:

mk rec {
  nix.frontmatter = mkDefaultFrontmatterWithReuseArgs (defaultReuseFrontmatterArgs // {
    licenseID = package.license;
  });
  package.name = "sel4-sync";
  package.license = "MIT";
  dependencies = {
    inherit (versions) lock_api;
    inherit (localCrates)
      sel4
      sel4-immediate-sync-once-cell
      sel4-sync-abstractions
    ;
  };
}
