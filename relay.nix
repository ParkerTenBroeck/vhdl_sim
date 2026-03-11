{ pkgs ? import <nixpkgs> {} }:

pkgs.rustPlatform.buildRustPackage {
  pname = "relay";
  version = "0.1.0";
  src = ./.;

  cargoLock = {
    lockFile = ./Cargo.lock;
  };

  cargoBuildFlags = [ "-p" "relay" ];

  nativeBuildInputs = [ pkgs.makeWrapper ];
  buildInputs = [ pkgs.ghdl-llvm ];

  postFixup = ''
    wrapProgram $out/bin/relay \
      --prefix PATH : ${pkgs.lib.makeBinPath [ pkgs.ghdl-llvm ]}
  '';
}
