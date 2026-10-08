{
  description = "haru — save clipboard images to files (pngpaste alternative)";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  outputs =
    { self, nixpkgs }:
    let
      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];
      forAllSystems = f: nixpkgs.lib.genAttrs systems (system: f nixpkgs.legacyPackages.${system});
    in
    {
      packages = forAllSystems (pkgs: rec {
        haru = pkgs.rustPlatform.buildRustPackage {
          pname = "haru";
          version = (builtins.fromTOML (builtins.readFile ./Cargo.toml)).package.version;

          src = self;
          cargoLock.lockFile = ./Cargo.lock;

          meta = with pkgs.lib; {
            description = "Save an image from the clipboard to a file (pngpaste alternative)";
            homepage = "https://github.com/ruaylabs/haru";
            license = licenses.mit;
            mainProgram = "haru";
            platforms = platforms.unix;
          };
        };
        default = haru;
      });

      apps = forAllSystems (pkgs: rec {
        haru = {
          type = "app";
          program = "${self.packages.${pkgs.system}.haru}/bin/haru";
        };
        default = haru;
      });

      overlays.default = final: prev: {
        haru = self.packages.${final.system}.haru;
      };
    };
}
