#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ lib, stdenv, buildPlatform, hostPlatform, buildPackages
, runCommand, runCommandCC, linkFarm, writeText
, crateUtils
, defaultRustEnvironment, defaultRustTargetTriple
}:

{ rustEnvironment ? defaultRustEnvironment
, targetTriple ? defaultRustTargetTriple
, release ? true
, profile ? if release then "release" else "dev"
, alloc ? true
, std ? false
, compilerBuiltinsMem ? true
, compilerBuiltinsC ? rustEnvironment.compilerRTSource != null
, src ? null
, extraManifest ? {}
, extraConfig ? {}
}:

let
  workspace = linkFarm "workspace" [
    { name = "Cargo.toml"; path = manifest; }
    { name = "Cargo.lock"; path = lockfile; }
  ];

  package = {
    name = "dummy";
    version = "0.0.0";
  };

  manifest = crateUtils.toTOMLFile "Cargo.toml" (crateUtils.clobber [
    {
      inherit package;
      lib.path = crateUtils.dummyInSrc "lib.rs" (writeText "main.rs" ''
        #![no_std]
      '');
    }
    extraManifest
  ]);

  lockfile = crateUtils.toTOMLFile "Cargo.lock" {
    package = [
      package
    ];
  };

  config = crateUtils.toTOMLFile "config" (crateUtils.clobber [
    # baseConfig # TODO will trigger rebuild
    {
      target = {
        "${targetTriple.name}" = {
          rustflags = [
            "--sysroot" "/dev/null"
            "-C" "embed-bitcode=yes"
            # "-C" "force-unwind-tables=yes" # TODO compare with "requires-uwtable" in target.json
          ];
        };
      };
    }
    rustEnvironment.vendoredSysrootLockfile.configFragment
    extraConfig
  ]);

  crates = lib.concatStringsSep "," ([
    "core"
    "compiler_builtins"
  ] ++ lib.optionals alloc [
    "alloc"
  ] ++ lib.optionals std [
    "std"
  ]);

  features = lib.concatStringsSep "," (lib.flatten [
    (lib.optional compilerBuiltinsMem "compiler-builtins-mem")
    (lib.optional compilerBuiltinsC "compiler-builtins-c")
  ]);

in
(if compilerBuiltinsC then runCommandCC else runCommand) "sysroot" ({
  depsBuildBuild = [ buildPackages.stdenv.cc ];
  nativeBuildInputs = [ rustEnvironment.rustToolchain ];
  RUST_TARGET_PATH = rustEnvironment.mkTargetPath targetTriple;
} // lib.optionalAttrs (!rustEnvironment.isNightly) {
  # HACK
  RUSTC_BOOTSTRAP = 1;
} // lib.optionalAttrs compilerBuiltinsC {
  "CC_${targetTriple.name}" = "${stdenv.cc.targetPrefix}gcc";
  RUST_COMPILER_RT_ROOT = rustEnvironment.compilerRTSource;
} // lib.optionalAttrs (src != null) {
  # HACK
  __CARGO_TESTS_ONLY_SRC_ROOT = src;
}) ''
  cargo build \
    -Z unstable-options \
    --offline \
    --frozen \
    --config ${config} \
    ${lib.optionalString (profile != null) "--profile ${profile}"} \
    --target ${targetTriple.name} \
    -Z build-std=${crates} \
    -Z build-std-features=${features} \
    --manifest-path ${workspace}/Cargo.toml \
    --target-dir $(pwd)/target

  d=$out/lib/rustlib/${targetTriple.name}/lib
  mkdir -p $d
  mv target/${targetTriple.name}/*/deps/* $d
''

# TODO
# rel=lib/rustlib/${buildPlatform.config}/bin
# d=$out/$rel
# mkdir -p $d
# ln -s ${rustToolchain}/$rel/* $d

# NOTE "-Z avoid-dev-deps" for deps of std
