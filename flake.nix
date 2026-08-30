{
  description = "RL2.ModLoader database interaction service. Used by rl2-modloader.pages.dev";

  inputs = {
    unstable.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs = { self, unstable } : let
    systems = [ "x86_64-linux" "aarch64-linux"];
    lib = unstable.lib;
    manifest = builtins.fromTOML (builtins.readFile ./Cargo.toml);
    forEachSystem = func: lib.foldAttrs (item: acc: item // acc) {} (lib.map func systems);
  in forEachSystem (system: let
      pkgs = unstable.legacyPackages.${system};
    in lib.recursiveUpdate {
      packages.${system} = rec {
        default = rl2db;
      
        rl2db = pkgs.rustPlatform.buildRustPackage {
          inherit (manifest.package) name version;
          src = lib.cleanSource ./.;
          cargoLock.lockFile = ./Cargo.lock;
        };

        data = pkgs.stdenvNoCC.mkDerivation {
          name = "rl2db-data";
          src = lib.cleanSource ./.;
          version = rl2db.version;

          buildPhase = ''
            mkdir -p $out
            cp -r assets $out
            cp -r static $out
            cp mods.db $out
          '';
        }
      };
      devShell.${system} = pkgs.mkShell {
        buildInputs = [ 
            pkgs.cargo pkgs.cargo-watch
            pkgs.rustc 
            pkgs.rust-analyzer 
            pkgs.rustfmt 
            pkgs.sqlite pkgs.rlwrap
        ];
        shellHook = ''
          echo "Entered RL2.DB development shell"
        '';
      };
    } {
      packages.mips.default = let pkgs = import unstable {
        system = "x86_64-linux";
        crossSystem = { config = "mipsel-unknown-linux-musl"; };
      }; in pkgs.rustPlatform.buildRustPackage {
          inherit (manifest.package) name version;
          src = lib.cleanSource ./.;
          cargoLock.lockFile = ./Cargo.lock;
          env.RUSTFLAGS = "-C target-feature=+crt-static";
      };
    }
  );
}
