{ makeRustPlatform, toolchain }:
let
  rustPlatform = makeRustPlatform {
    cargo = toolchain;
    rustc = toolchain;
  };
in
rustPlatform.buildRustPackage {
  pname = "tmux-mvr";
  version = "0.0.3";

  src = ../.;
  cargoLock.lockFile = ../Cargo.lock;
}
