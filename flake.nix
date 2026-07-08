{
  description = "Navix client flake";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
  };

  
  
  outputs = { self, nixpkgs }:
  let
    system = "x86_64-linux";
    pkgs = import nixpkgs { inherit system; };
    nativeBuildInputs = with pkgs; [
      cargo
      rustc
      rust-analyzer
      tpm2-tools
      evcxr
      openssl
      pkg-config
    ];
  in 
  {
    devShells.${system}.default = pkgs.mkShell {
      name = "rust dev";
      inherit nativeBuildInputs; 
    };

  };
}
