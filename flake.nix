{
  description = "UTPM is a package manager for local and remote Typst packages.";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";

    crane.url = "github:ipetkov/crane";

    flake-utils.url = "github:numtide/flake-utils";

    flake-compat.url = "https://flakehub.com/f/edolstra/flake-compat/1.tar.gz";
  };

  outputs =
    { self, ... }@inputs:
    {
      overlays.default = final: _prev: {
        utpm =
          let
            craneLib = inputs.crane.mkLib final;
            common = {
              src = ./.;
              strictDeps = true;
              buildInputs = [ final.openssl ];
              nativeBuildInputs = [ final.pkg-config ];
              OPENSSL_NO_VENDOR = 1;
            };
          in
          craneLib.buildPackage (
            common
            // {
              cargoArtifacts = craneLib.buildDepsOnly common;
              nativeCheckInputs = [ final.git ];
              nativeBuildInputs = common.nativeBuildInputs ++ [ final.makeWrapper ];
              # --suffix so a git the user already has on PATH still wins.
              postInstall = ''
                wrapProgram $out/bin/utpm --suffix PATH : ${final.lib.makeBinPath [ final.git ]}
              '';
              meta = {
                homepage = "https://github.com/typst-community/utpm";
                license = final.lib.licenses.mit;
                description = "UTPM is a package manager for local and remote Typst packages.";
                longDescription = "UTPM is a package manager for local and remote Typst packages. Quickly create and manage projects and templates on your system, and publish them directly to Typst Universe.";
                mainProgram = "utpm";
              };
            }
          );
      };
    }
    // inputs.flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import inputs.nixpkgs {
          inherit system;
          overlays = [ self.overlays.default ];
        };
      in
      {
        devShells.default = pkgs.mkShell {
          inputsFrom = [ self.packages.${pkgs.stdenv.hostPlatform.system}.default ];
          packages = [
            #lsp
            pkgs.rust-analyzer
          ];
        };

        packages = {
          utpm = pkgs.utpm;
          default = pkgs.utpm;
        };

        # `pkg install` refuses to run without git, and the stdenv PATH has none.
        checks.finds-git = pkgs.runCommand "utpm-finds-git" { nativeBuildInputs = [ pkgs.utpm ]; } ''
          export HOME=$TMPDIR/home
          export UTPM_DATA_PATH=$TMPDIR/data
          export TYPST_PACKAGE_PATH=$TMPDIR/packages
          export TYPST_PACKAGE_CACHE_PATH=$TMPDIR/cache
          mkdir -p $HOME project
          cd project
          cat > typst.toml <<EOT
          [package]
          name = "my-pkg"
          version = "0.1.0"
          entrypoint = "main.typ"
          EOT
          touch main.typ

          utpm pkg install .

          test -f $TYPST_PACKAGE_PATH/local/my-pkg/0.1.0/typst.toml
          touch $out
        '';
      }
    );
}
