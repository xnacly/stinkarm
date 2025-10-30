{
  description = "stinkarm — ARMv7 userspace binary emulator for x86 linux systems";

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
            rust-analyzer
            gdb
            gcc-arm-embedded
            binutils
            qemu
          ];

          shellHook = ''
            if ! command -v rustc >/dev/null 2>&1; then
              echo "Installing rustup toolchain..."
              curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
              export PATH="$HOME/.cargo/bin:$PATH"
            fi

            rustup default stable

            rustup target add armv7-unknown-linux-gnueabi

            echo "Welcome to stinkarm dev shell (ARMv7 cross-compilation ready)"
          '';
        };
      });
}
