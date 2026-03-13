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
  buildInputs = [ pkgs.ghdl-llvm pkgs.verilator pkgs.python3 pkgs.zlib ];

  postFixup = ''
    wrapProgram $out/bin/relay \
      --prefix PATH : ${pkgs.lib.makeBinPath [ pkgs.ghdl-llvm pkgs.verilator pkgs.python3 pkgs.glib.dev ]} \
      --prefix LIBRARY_PATH : ${pkgs.lib.makeLibraryPath [ pkgs.zlib ]} \
      --prefix LD_LIBRARY_PATH : ${pkgs.lib.makeLibraryPath [ pkgs.zlib ]}
  '';
}
