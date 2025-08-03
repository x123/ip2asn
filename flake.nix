{
  description = "A Nix-flake-based Rust development environment";

  inputs = {
    nixpkgs.url = "nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = inputs: let
    supportedSystems = ["x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin"];
    forEachSupportedSystem = f:
      inputs.nixpkgs.lib.genAttrs supportedSystems (system:
        f {
          pkgs = import inputs.nixpkgs {
            inherit system;
            overlays = [
              inputs.rust-overlay.overlays.default
              inputs.self.overlays.default
            ];
          };
        });
  in {
    overlays.default = final: prev: {
      rustToolchain = let
        rust = prev.rust-bin;
      in
        if builtins.pathExists ./rust-toolchain.toml
        then rust.fromRustupToolchainFile ./rust-toolchain.toml
        else if builtins.pathExists ./rust-toolchain
        then rust.fromRustupToolchainFile ./rust-toolchain
        else
          rust.stable.latest.default.override {
            extensions = ["clippy" "rust-src" "rustfmt" "llvm-tools-preview"];
          };
    };

    devShells = forEachSupportedSystem ({pkgs}: {
      default = pkgs.mkShell {
        packages = with pkgs; [
          cargo-deny
          cargo-edit
          cargo-watch
          just
          openssl
          pkg-config
          rust-analyzer
          rustToolchain
          shellcheck
        ];

        env = {
          # Required by rust-analyzer
          RUST_SRC_PATH = "${pkgs.rustToolchain}/lib/rustlib/src/rust/library";
          CARGO_HOME = "./.cargo";
        };

        shellHook = ''
          export PATH="$PWD/bin:$PWD/.cargo/bin:$PATH"
        '';
      };
    });

    packages = forEachSupportedSystem ({pkgs}: {
      default = pkgs.rustPlatform.buildRustPackage {
        pname = "ip2asn";
        version = "0.1.0";
        src = ./.;
        cargoLock = {
          lockFile = ./Cargo.lock;
        };
        nativeBuildInputs = with pkgs; [
          pkg-config
        ];
        buildInputs = with pkgs; [
          openssl
        ];
        meta = with pkgs.lib; {
          description = "A high-performance, memory-efficient Rust crate for mapping IP addresses to Autonomous System (AS) information";
          homepage = "https://github.com/x123/ip2asn";
          license = licenses.mit;
          maintainers = [ maintainers.x123 ];
        };
      };
    });
  };
}
