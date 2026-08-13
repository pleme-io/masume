{
  description = "Masume (升目) — the ruled squares of a page. A terminal substrate whose VT dispatch table, conformance matrix, terminfo entry and docs are all EMITTED from one typed sequence catalog rather than hand-written, so two faces cannot disagree about a table neither of them writes. Naturalizes the essence of Ghostty / kitty / WezTerm / Alacritty / contour rather than vendoring any of them. Theory — theory/NATURALIZE-TERMINAL.md. Name ratified 2026-08-13, opens The Page 頁.";
  inputs = {
    nixpkgs = {
      follows = "substrate/nixpkgs";
    };
    flake-utils = {
      url = "github:numtide/flake-utils";
    };
    substrate = {
      url = "github:pleme-io/substrate";
    };
  };
  outputs = inputs @ { self, nixpkgs, crate2nix, flake-utils, substrate, devenv, ... }:
    (import "${substrate}/lib/rust-workspace-release-flake.nix" {
      inherit nixpkgs crate2nix flake-utils devenv;
    }) {
      toolName = "masume";
      packageName = "masume";
      src = self;
      repo = "pleme-io/masume";
    };
}
