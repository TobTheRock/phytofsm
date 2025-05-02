{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  name = "rust-plantuml-env";

  # Rust development environment
  buildInputs = [
    pkgs.rustup          # Rust toolchain manager
    pkgs.cargo           # Rust package manager
    pkgs.rustc           # Rust compiler
    pkgs.plantuml        # PlantUML tool
    pkgs.openjdk         # Java runtime for PlantUML
    pkgs.bat             # Previewer for PlantUML files
  ];

  # Set up rustup default toolchain
  shellHook = ''
    rustup default stable
    echo "Rust and PlantUML environment loaded."
  '';
}
