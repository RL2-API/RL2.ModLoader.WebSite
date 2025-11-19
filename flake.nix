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
    in {
      packages.${system} = rec {
        default = rl2db;
      
        rl2db = pkgs.rustPlatform.buildRustPackage {
          inherit (manifest.package) name version;
          src = lib.cleanSource ./.;
          cargoLock.lockFile = ./Cargo.lock;
        };
      };
      devShell.${system} = pkgs.mkShell {
        buildInputs = [ pkgs.cargo pkgs.rustc ];
        shellHook = ''
          echo "Entered RL2.DB development shell"
        '';
      };
    }
  );
}
