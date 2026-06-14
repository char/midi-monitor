{
  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  outputs = { nixpkgs, ... }:
    let
      systems = [ "x86_64-linux" ];
      forEachSystem = f:
        nixpkgs.lib.genAttrs systems (system: f nixpkgs.legacyPackages.${system});
    in
    {
      devShells = forEachSystem (pkgs: {
        default = pkgs.mkShell {
          packages = with pkgs; [
            cargo
            rustc
            rustfmt
            clippy
            pkg-config
            clang
            lld
          ];

          buildInputs = with pkgs; [
            alsa-lib
            fontconfig
            freetype
            libxkbcommon
            vulkan-loader
            wayland
            libx11
            libxcursor
            libxi
            libxrandr
            libxcb
          ];

          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath (with pkgs; [
            alsa-lib
            fontconfig
            freetype
            libxkbcommon
            vulkan-loader
            wayland
            libx11
            libxcursor
            libxi
            libxrandr
            libxcb
          ]);
        };
      });
    };
}
