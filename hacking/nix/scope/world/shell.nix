#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ lib, stdenv, buildPackages
, writeText, emptyFile
, mkShell
, defaultRustToolchain
, defaultRustTargetTriple
, bareMetalRustTargetTriple
, libclangPath
, sources
, dummyCapDLSpec, serializeCapDLSpec
, seL4RustEnvVars
, mkSeL4RustTargetTriple
, worldConfig
, seL4ForBoot
, crateUtils

, hostPlatform
, cmake
, perl
, python3Packages
}:

let
  kernelLoaderConfigEnvVars = lib.optionalAttrs (!worldConfig.isMicrokit && worldConfig.kernelLoaderConfig != null) {
    SEL4_KERNEL_LOADER_CONFIG = writeText "loader-config.json" (builtins.toJSON worldConfig.kernelLoaderConfig);
  };

  capdlEnvVars = lib.optionalAttrs (!worldConfig.isMicrokit) {
    CAPDL_SPEC_FILE = serializeCapDLSpec { inherit (dummyCapDLSpec) cdl; };
    CAPDL_FILL_DIR = dummyCapDLSpec.fill;
  };

  libcDir = "${stdenv.cc.libc}/${hostPlatform.config}";

  bindgenEnvVars =
    let
      targets = lib.flatten (
        lib.forEach [ true false ] (microkit:
          lib.forEach [ true false ] (minimal:
            mkSeL4RustTargetTriple { inherit microkit minimal; })));
    in lib.listToAttrs (lib.forEach targets (target: {
      name = "BINDGEN_EXTRA_CLANG_ARGS_${target.name}";
      value = [ "-I${libcDir}/include" ];
    }));

  miscEnvVars = {
    CHILD_ELF = emptyFile;
  };

in
mkShell (seL4RustEnvVars // kernelLoaderConfigEnvVars // capdlEnvVars // bindgenEnvVars // miscEnvVars // {
  # TODO
  RUST_SEL4_TARGET = defaultRustTargetTriple.name;

  RUST_BARE_METAL_TARGET = bareMetalRustTargetTriple.name;

  HOST_CARGO_FLAGS = lib.concatStringsSep " " [
    "-Z" "build-std=core,alloc,compiler_builtins"
    "-Z" "build-std-features=compiler-builtins-mem"
  ];

  HOST_CC = "${buildPackages.stdenv.cc.targetPrefix}gcc";

  LIBCLANG_PATH = libclangPath;

  hardeningDisable = [ "all" ];

  nativeBuildInputs = [
    defaultRustToolchain
    cmake
    perl
    python3Packages.jsonschema
    python3Packages.jinja2
  ];

  depsBuildBuild = [
    buildPackages.stdenv.cc
  ];

  shellHook = ''
    export h=$HOST_CARGO_FLAGS
    export t="--target $RUST_SEL4_TARGET"
    export bt="--target $RUST_BARE_METAL_TARGET"
  '';
})
