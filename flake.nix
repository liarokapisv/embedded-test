{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    fenix = {
      url = "github:nix-community/fenix/monthly";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    naersk = {
      url = "github:nix-community/naersk";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, fenix, naersk, ... }:
    let
      forAllSystems = nixpkgs.lib.genAttrs [
        "x86_64-linux"
        "aarch64-linux"
      ];
      target = "thumbv7em-none-eabihf";
      mkToolchain = system: with fenix.packages.${system}; combine [
        latest.cargo
        latest.rustc
        latest.clippy
        latest.llvm-tools
        latest.rustfmt
        targets.${target}.latest.rust-std
      ];
    in
    {
      packages = forAllSystems (system: {
        default =
          let
            toolchain = mkToolchain system;
            buildRustPackage =
              (naersk.lib.${system}.override {
                cargo = toolchain;
                rustc = toolchain;
              }).buildPackage;
          in
          buildRustPackage {
            pname = "artemis";
            version = "1.0.0";
            src = ./.;
            CARGO_BUILD_TARGET = target;
          };
      });

      devShells = forAllSystems (system:
        let
          pkgs = nixpkgs.legacyPackages.${system};
        in
        {
          default = pkgs.mkShell {
            inputsFrom = [
              self.packages.${system}.default
            ];
            packages = [
              pkgs.probe-rs
            ];
          };

        });
    };
}
