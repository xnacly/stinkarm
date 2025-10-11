{
  description = "stinkarm — a stinky ARM emulator";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
      in {
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            rustc
            cargo
            rust-analyzer
            gdb
            gcc-arm-embedded
            binutils
            qemu
          ];

          shellHook = ''
            echo "🦀 Welcome to stinkarm dev shell"
            echo "Rust + ARM GCC ready"
          '';
        };
      });
}
