#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ lib
, hostPlatform
, buildPackages
, runCommand, runCommandCC, linkFarm, writeScript
, jq
, symlinkToRegularFile
, crateUtils
, mkSeL4
, mkMicrokit
, sources
}:

let
  elaborateWorldConfig =
    { isMicrokit ? false
    , kernelConfig ? null
    , kernelLoaderConfig ? null
    , microkitConfig ? null
    , canSimulate ? false
    , platformRequiresLoader ? true
    , mkPlatformSystemExtension ? _: { attrs = {}; links = []; }
    , ...
    } @ args:
    args // {
      inherit
        isMicrokit
        kernelConfig
        kernelLoaderConfig
        microkitConfig
        canSimulate
        platformRequiresLoader
        mkPlatformSystemExtension
      ;
    };
in

unelaboratedWorldConfig:

let
  worldConfig = elaborateWorldConfig unelaboratedWorldConfig;
in

self: with self;

{
  inherit worldConfig;

  microkit = assert worldConfig.isMicrokit; mkMicrokit worldConfig.microkitConfig;

  microkitForUserspace = microkit;
  microkitForBoot = microkit;

  microkitDir =
    let
      inherit (worldConfig.microkitConfig) board config;
    in
      "${microkitForUserspace.sdk}/board/${board}/${config}";

  seL4 = assert !worldConfig.isMicrokit; mkSeL4 worldConfig.kernelConfig;

  seL4ForUserspace = seL4;

  seL4ForBoot = seL4.overrideAttrs (_: {
    # src = lib.cleanSource (sources.localRoot + "/seL4");
  });

  seL4IncludeDir =
    if worldConfig.isMicrokit
    then
      "${microkitDir}/include"
    else
      "${seL4ForUserspace}/libsel4/include";

  seL4RustEnvVars =
    if worldConfig.isMicrokit
    then {
      SEL4_INCLUDE_DIRS = "${microkitDir}/include";
    }
    else {
      SEL4_PREFIX = seL4ForUserspace;
    };

  seL4Modifications = crateUtils.elaborateModifications {
    modifyDerivation = drv: drv.overrideAttrs (self: super: seL4RustEnvVars);
  };

  kernelBinary = assert !worldConfig.isMicrokit; "${seL4ForBoot}/bin/kernel.elf";

  kernelBinary32Bit =
    assert hostPlatform.isx86_64;
    runCommandCC "kernel32.elf" {} ''
      $OBJCOPY -O elf32-i386 ${kernelBinary} $out
    '';

  libsel4 = if worldConfig.isMicrokit then microkitDir else "${seL4ForUserspace}/libsel4";

  seL4Config =
    let
      f = seg: builtins.fromJSON (builtins.readFile (symlinkToRegularFile "${seg}-gen_config.json" "${seL4IncludeDir}/${seg}/gen_config.json"));
    in
      f "kernel" // f "sel4";

  seL4ConfigJSON = runCommand "sel4-config.json" {
    nativeBuildInputs = [
      jq
    ];
    json = builtins.toJSON seL4Config;
    passAsFile = [ "json" ];
  } ''
    jq . $jsonPath > $out
  '';

  mkTask = callPackage ./mk-task.nix {};

  sel4-capdl-initializer = callPackage ./capdl/sel4-capdl-initializer.nix {};
  objectSizes = callPackage ./capdl/object-sizes.nix {};
  mkSmallCapDLInitializer = callPackage ./capdl/mk-small-capdl-initializer.nix {};
  serializeCapDLSpec = callPackage ./capdl/serialize-capdl-spec.nix {};
  dummyCapDLSpec = callPackage ./capdl/dummy-capdl-spec.nix {};
  mkSimpleCompositionCapDLSpec = callPackage ./capdl/mk-simple-composition-capdl-spec.nix {};
  mkCapDLInitializerWithSpec = callPackage ./capdl/mk-capdl-initializer-with-spec.nix {};

  mkCapDLInitializer =
    { spec ? null
    , specAttrs ? spec.specAttrs
    , small ? false
    , extraDebuggingLinks ? []
    }:

    lib.fix (self:
      (if small then mkSmallCapDLInitializer else mkCapDLInitializerWithSpec) spec.specAttrs // {
        inherit spec;
        debuggingLinks = [
          { name = "spec.cdl"; path = spec.specAttrs.cdl; }
          { name = "fill"; path = spec.specAttrs.fill; }
        ] ++ lib.optionals (spec != null) [
          { name = "spec"; path = spec; }
        ] ++ lib.optionals (!small) [
          { name = "initializer.full.elf"; path = self.split.full; }
        ] ++ extraDebuggingLinks;
      }
    );

  sel4-kernel-loader = callPackage ./sel4-kernel-loader.nix {
    inherit (worldConfig) kernelLoaderConfig;
  };

  mkSeL4KernelLoaderWithPayload = { appELF }: callPackage ./mk-sel4-kernel-loader-with-payload.nix {} {
    app = appELF;
  };

  mkSystem =
    { rootTask
    , extraDebuggingLinks ? []
    , passthru ? {}
    }:
    let
      loader =
        assert worldConfig.platformRequiresLoader;
        mkSeL4KernelLoaderWithPayload {
          appELF = rootTask.elf;
        };

      symbolizeRootTaskBacktrace = writeScript "x.sh" ''
        #!${buildPackages.runtimeShell}
        exec ${buildPackages.this.sel4-backtrace-cli}/bin/sel4-symbolize-backtrace -f ${rootTask.elf} "$@"
      '';
    in {
      inherit loader rootTask;
      rootTaskImage = rootTask.elf;
      loaderImage = loader.elf;
      debuggingLinks = [
        { name = "kernel.elf"; path = "${seL4ForBoot}/bin/kernel.elf"; }
        { name = "root-task.elf"; path = rootTask.elf; }
        { name = "symbolize-root-task-backtrace"; path = symbolizeRootTaskBacktrace; }
        { name = "sel4-symbolize-backtrace";
          path = "${buildPackages.this.sel4-backtrace-cli}/bin/sel4-symbolize-backtrace";
        }
      ] ++ lib.optionals worldConfig.platformRequiresLoader [
        { name = "loader.elf"; path = loader.elf; }
        { name = "loader.debug.elf"; path = loader.split.full; }
      ] ++ (rootTask.debuggingLinks or []) ++ extraDebuggingLinks;
    } // passthru;

  callPlatform =
    { system
    , extraPlatformArgs ? {}
    , extraDebuggingLinks ? []
    , ...
    } @ args:
    let
      platformSystemExtension = worldConfig.mkPlatformSystemExtension ({
        inherit worldConfig;
      } // (
        if worldConfig.platformRequiresLoader
        then { inherit (system) loaderImage; }
        else { inherit (system) rootTaskImage; }
      ) // extraPlatformArgs);
    in {
      links = linkFarm "links" (
        system.debuggingLinks ++ platformSystemExtension.links
      );
    }
    // platformSystemExtension.attrs
    // builtins.removeAttrs system [ "debuggingLinks" ]
    // builtins.removeAttrs args [ "extraPlatformArgs" "extraDebuggingLinks" ];

  instances = callPackage ./instances {};

  docs = callPackage ./docs.nix {};

  shell = callPackage ./shell.nix {};
}
