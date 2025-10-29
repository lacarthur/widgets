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
        wayland
      ];
      nativeBuildInputs = with pkgs; [
        pkg-config
      ];
      cargoLock = {
        lockFile = ./Cargo.lock;
        outputHashes = {
          "accesskit-0.12.2" = "sha256-ksaYMGT/oug7isQY8/1WD97XDUsX2ShBdabUzxWffYw=";
          "clipboard_macos-0.1.0" = "sha256-temNg+RdvquSLAdkwU5b6dtu9vZkXjnDASS/eJo2rz8=";
          "cosmic-text-0.11.2" = "sha256-Jpgbg1DScteec7ItcGgbQYXu1bBNYJEw1SGsxpcxYfM=";
          "iced-0.12.0" = "sha256-/8qxBL9yq3yAkQE8XWSUWIh6pVLgdYuDEVzFjIIFtp0=";
          "smithay-client-toolkit-0.18.0" = "sha256-/7twYMt5/LpzxLXAQKTGNnWcfspUkkZsN5hJu7KaANc=";
          "smithay-clipboard-0.8.0" = "sha256-phySYRO6z18X5kB1CZ5/+AYwzU8ooQ+BuOvBeyuIfXw=";
          "softbuffer-0.4.1" = "sha256-a0bUFz6O8CWRweNt/OxTvflnPYwO5nm6vsyc/WcXyNg=";
        };
      };
    };
  };
}
