{ lib, buildPlatform, hostPlatform
, writeText, linkFarm
, toTOMLFile
, chooseLinkerForRustTarget ? throw "chooseLinkerForRustTarget must be set"
}:

let
  compose = f: g: x: f (g x);
in

rec {

  inherit toTOMLFile;

  clobber = lib.fold lib.recursiveUpdate {};

  ###

  dummyLibInSrc = dummyInSrc "lib.rs" dummyLib;
  dummyMainWithStdInSrc = dummyInSrc "main.rs" dummyMainWithStd;
  dummyMainWithOrWithoutStdInSrc = dummyInSrc "main.rs" dummyMainWithOrWithoutStdInSrc;

  dummyInSrc = name: path: 
    let
      src = linkFarm "dummy-src" [
        {
          inherit name path;
        }
      ];
    in
      "${src}/${name}";

  dummyMainWithStd = writeText "main.rs" ''
    fn main() {}
  '';

  dummyLib = writeText "lib.rs" ''
    #![no_std]
  '';

  dummyMainWithOrWithoutStd = writeText "main.rs" ''
    #![cfg_attr(target_os = "none", no_std)]
    #![cfg_attr(target_os = "none", no_main)]
    #![cfg_attr(target_os = "none", feature(lang_items))]

    #[cfg(target_os = "none")]
    #[panic_handler]
    extern fn panic_handler(_: &core::panic::PanicInfo) -> ! {
      todo!()
    }

    #[cfg(target_os = "none")]
    #[lang = "eh_personality"]
    extern fn eh_personality() {
    }

    #[cfg(not(target_os = "none"))]
    fn main() {
    }
  '';

  ###

  getClosureOfCrate = root: root.closure;
  getClosureOfCrates = lib.foldl' (acc: crate: acc // getClosureOfCrate crate) {};

  collectReals = reals: collectRealsAndDummies reals [];

  collectDummies = dummies: collectRealsAndDummies [] dummies;

  collectRealsAndDummies = reals: dummies: linkFarm "crates" (map (crate: {
    name = crate.name;
    path = crate.real;
  }) reals ++ map (crate: {
    name = crate.name;
    path = crate.dummy;
  }) dummies);

  ###

  # TODO improve this mechanism
  linkerConfig = { rustToolchain, rustTargetName }@args:
    let
      f = { rustTargetName, platform }:
        let
          linker = chooseLinkerForRustTarget {
            inherit rustToolchain rustTargetName platform;
          };
        in
          lib.optionalAttrs (linker != null) {
            target = {
              "${rustTargetName}".linker = linker;
            };
          };
    in
      clobber [
        (f { rustTargetName = buildPlatform.config; platform = buildPlatform; })
        (f { inherit rustTargetName; platform = hostPlatform; })
      ];

  baseConfig = { rustToolchain, rustTargetName }@args: clobber [
    {
      build.incremental = false;
    }
    (linkerConfig args)
  ];

  ###

  defaultIntermediateLayer = {
    crates = [];
    modifications = {};
  };

  elaborateModifications =
    { modifyManifest ? lib.id
    , modifyConfig ? lib.id
    , modifyDerivation ? lib.id
    , extraCargoFlags ? []
    }:
    {
      inherit
        modifyManifest
        modifyConfig
        modifyDerivation
        extraCargoFlags
      ;
    };

  composeModifications = f: g: {
    modifyManifest = compose f.modifyManifest g.modifyManifest;
    modifyConfig = compose f.modifyConfig g.modifyConfig;
    modifyDerivation = compose f.modifyDerivation g.modifyDerivation;
    extraCargoFlags = f.extraCargoFlags ++ g.extraCargoFlags;
  };

  ###

  traverseAttrs = f: attrs: state0:
    let
      op = { acc, state }: { name, value }:
        let
          step = f name value state;
        in {
          acc = acc ++ [
            {
              inherit name;
              inherit (step) value;
            }
          ];
          inherit (step) state;
        };
      nul = {
        acc = [];
        state = state0;
      };
      final = lib.foldl' op nul (lib.mapAttrsToList lib.nameValuePair attrs);
    in {
      attrs = lib.listToAttrs final.acc;
      inherit (final) state;
    };

  # f :: (attrPath :: [String]) -> (dependencyName :: String) -> (pathValue :: String) -> (state :: a) -> { pathValue :: String, state :: a }
  traversePathDependencies = f: manifest: state0:
    let
      dependencyAttributes = [
        "dependencies"
        "dev-dependencies"
        "build-dependencies"
      ];

      traverseTheseDependencies = attrPath: dependencies:
        let
        in lib.flip traverseAttrs dependencies (name: value: state':
          if value ? "path"
          then
            let step = f attrPath name value.path state';
            in {
              value = value // { path = step.pathValue; };
              inherit (step) state;
            }
          else {
            inherit value;
            state = state';
          }
        );

      fForTarget = attrPath: name: value: state':
        if lib.elem name dependencyAttributes
        then
          let
            step = traverseTheseDependencies (attrPath ++ [ name ]) value state';
          in {
            value = step.attrs;
            state = step.state;
          }
        else {
          inherit value;
          state = state';
        }
      ;

      traverseTarget = attrPath: traverseAttrs (fForTarget attrPath);

      fForTargets = name: value: state':
        if name == "target"
        then
          let step =
            traverseAttrs
              (name: value: state':
                let step = traverseAttrs (fForTarget [ "target" name ]) value state';
                in {
                  value = step.attrs;
                  inherit (step) state;
                }
              )
              value
              state'
            ;
          in {
            inherit (step) state;
            value = step.attrs;
          }
        else {
          inherit value;
          state = state';
        }
      ;

      traverseTargets = traverseAttrs fForTargets;
    
      step = traverseTarget [] manifest state0;
    in
      traverseTargets step.attrs step.state;

  extractAndPatchPathDependencies = patch: manifest:
    let
      step = traversePathDependencies
        (attrPath: dependencyName: pathValue: state':
          let
            consistent = if state' ? dependencyName then state'.dependencyName == pathValue else true;
          in
            assert consistent;
            {
              pathValue = patch dependencyName pathValue;
              state = state' // { "${dependencyName}" = pathValue; };
            }
        )
        manifest
        {}
      ;
    in {
      patchedManifest = step.attrs;
      pathDependencies = step.state;
    };

  ###

  crateManifest = cratePath: builtins.fromTOML (builtins.readFile (crateManifestPath cratePath));
  crateManifestPath = cratePath: cratePath + "/Cargo.toml";
  crateSrcPath = cratePath: cratePath + "/src";

  mkCrate =
    cratePath:

    { extraPaths ? []
    }:

    let
      manifest = crateManifest cratePath;

      inherit (manifest.package) name;

      hasImplicitBuildScript = builtins.pathExists (cratePath + "/build.rs");
      hasExplicitBuildScript = manifest.package ? "build";
      hasAnyBuildScript = hasImplicitBuildScript || hasExplicitBuildScript;

      extractedAndPatched = extractAndPatchPathDependencies (dependencyName: pathValue: "../${dependencyName}") manifest;
      pathDependencies = extractedAndPatched.pathDependencies;
      manifestWithPatchedPathDependencies = extractedAndPatched.patchedManifest;

      realPatchedManifest = manifestWithPatchedPathDependencies;
  
      dummyPatchedManifest = clobber [
        manifestWithPatchedPathDependencies
        {
          lib.path = dummyLibInSrc;
        }
        (lib.optionalAttrs hasAnyBuildScript {
          package.build = dummyMainWithStdInSrc;
        })
      ];

      real = linkFarm "real-crate-${name}" ([
        (rec {
          name = "Cargo.toml";
          path = toTOMLFile name realPatchedManifest;
        })
        { name = "src";
          path = crateSrcPath cratePath;
        }
      ] ++ (lib.optionals hasImplicitBuildScript [
        { name = "build.rs";
          path = cratePath + "/build.rs";
        }
      ]) ++ (map
        (path: {
          name = path;
          path = cratePath + "/${path}";
        })
        extraPaths
      ));

      dummy = linkFarm "dummy-crate-${name}" [
        (rec {
          name = "Cargo.toml";
          path = toTOMLFile name dummyPatchedManifest;
        })
      ];

    in {
      inherit name real dummy pathDependencies;
    };

  augmentCrates = crates: lib.fix (selfCrates:
    lib.flip lib.mapAttrs crates (_: crate:
      let
        pathDependenciesList = lib.mapAttrsToList (dependencyName: _: selfCrates.${dependencyName}) crate.pathDependencies;
      in
        lib.fix (selfCrate: crate // {
          closure = {
            "${crate.name}" = selfCrate;
          } // lib.foldl' (acc: crate': acc // crate'.closure) {} pathDependenciesList;
        })
    )
  );

}
