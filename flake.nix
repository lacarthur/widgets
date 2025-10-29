{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
  };

  outputs = { self, nixpkgs}: let 
    pkgs = nixpkgs.legacyPackages."x86_64-linux";
  in {
    devShells."x86_64-linux".default = pkgs.mkShell rec {
      packages = with pkgs; [
        cargo
        rustc
        rustfmt
        clippy
        rust-analyzer
      ];

      buildInputs = with pkgs; [
        libxkbcommon
        pkg-config
        wayland
      ];

      env.RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
      LD_LIBRARY_PATH = builtins.foldl' (a: b: "${a}:${b}/lib") "${pkgs.vulkan-loader}/lib" buildInputs;
    };

    packages."x86_64-linux".default = pkgs.rustPlatform.buildRustPackage {
      name = "widgets";
      src = ./.;
      buildInputs = with pkgs; [
        libxkbcommon
        pkg-config
        wayland
      ];
      cargoLock.lockFile = ./Cargo.lock;
    };
  };
}
