{ pkgs ? import <nixpkgs> { } }:
pkgs.mkShell rec {
  buildInputs = with pkgs; [
    clippy rustfmt
    gcc cargo rustc
    pkg-config
  ];
  LD_LIBRARY_PATH = "${pkgs.lib.makeLibraryPath buildInputs}";
}
