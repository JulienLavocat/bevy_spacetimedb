{
  description = "Bevy SpacetimeDB WASM development environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "rust-analyzer" ];
          targets = [ "wasm32-unknown-unknown" ];
        };
      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            rustToolchain
            pkg-config

            # Bevy dependencies
            udev
            alsa-lib
            vulkan-loader

            # X11 dependencies
            xorg.libX11
            xorg.libXcursor
            xorg.libXi
            xorg.libXrandr

            # Wayland dependencies
            libxkbcommon
            wayland

            # WASM tools
            wasm-bindgen-cli
            wasm-pack
            binaryen

            # Additional tools
            nodejs
            trunk
          ];

          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath [
            pkgs.udev
            pkgs.alsa-lib
            pkgs.vulkan-loader
            pkgs.xorg.libX11
            pkgs.xorg.libXcursor
            pkgs.xorg.libXi
            pkgs.xorg.libXrandr
            pkgs.libxkbcommon
            pkgs.wayland
          ];

          shellHook = ''
            echo "Bevy SpacetimeDB WASM development environment"
            echo "Rust version: $(rustc --version)"
            echo "Cargo version: $(cargo --version)"
          '';
        };
      }
    );
}
