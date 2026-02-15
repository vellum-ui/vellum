{
  description = "Node.js native module in Rust using napi-rs";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
      in
      {
        devShells.default = pkgs.mkShell {
          nativeBuildInputs = with pkgs; [
            pkg-config
          ];

          RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";

          buildInputs = with pkgs; [
            nodejs
            rustc
            cargo
            rustfmt
            rust-analyzer
            clippy
            
            libclang
            sqlite

            # Common build dependencies
            openssl
            python3

            # Graphics / Windowing dependencies for Blitz
            pkg-config
            fontconfig
            freetype
            libX11
            libXcursor
            libXrandr
            libXi
            libxcb
            libxkbcommon
            wayland
            vulkan-loader
            libGL
          ];

          shellHook = ''
            echo "Environment for napi-rs development loaded"
            echo "Node: $(node --version)"
            echo "Rust: $(rustc --version)"

            export LIBCLANG_PATH="${pkgs.libclang.lib}/lib"

            # Configure FONTCONFIG_FILE to include system fonts
            # This is crucial for GUI apps to find fonts in a pure shell
            export FONTCONFIG_FILE=${pkgs.makeFontsConf { fontDirectories = [
              pkgs.dejavu_fonts
              pkgs.liberation_ttf
              pkgs.noto-fonts
              pkgs.noto-fonts-color-emoji
            ]; }}

            # Set LD_LIBRARY_PATH so that winit/tao/wgpu can find system libraries
            export LD_LIBRARY_PATH=${
              pkgs.lib.makeLibraryPath (
                with pkgs;
                [
                  libxkbcommon
                  vulkan-loader
                  libGL
                  wayland
                  libX11
                  libXcursor
                  libXrandr
                  libXi
                ]
              )
            }:$LD_LIBRARY_PATH
          '';
        };
      }
    );
}
