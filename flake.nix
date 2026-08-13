{
  description = "Masume (升目) — the ruled squares of a page. A terminal substrate whose VT dispatch table, conformance matrix, terminfo entry and docs are all EMITTED from one typed sequence catalog rather than hand-written, so two faces cannot disagree about a table neither of them writes. Naturalizes the essence of Ghostty / kitty / WezTerm / Alacritty / contour rather than vendoring any of them. Theory — theory/NATURALIZE-TERMINAL.md. Name ratified 2026-08-13, opens The Page 頁.";
  # HAND-EDITED over the repo-forge render, 2026-08-13.
  #
  # As generated, the outputs lambda destructured `crate2nix` and `devenv`
  # while `inputs` declared neither. Nix then tries to resolve an
  # undeclared outputs parameter through the flake REGISTRY, so
  # `nix flake check` died on `cannot find flake 'flake:crate2nix'` and CI
  # could never have gone green.
  #
  # `crate2nix` is now declared (the shape `pleme-io/tear` has shipped for
  # months); `devenv` is dropped from both the lambda and the `inherit`,
  # because substrate's own helper takes it as `devenv ? null` and its
  # documented call site does not pass it. An optional argument is not a
  # reason to acquire a dependency.
  #
  # Fixed at the generator too — `repo-forge-render/src/flake.rs` built the
  # inputs list and the outputs lambda from two independent lists with
  # nothing checking they agree, which is why `calha` carries the identical
  # defect.
  inputs = {
    nixpkgs = {
      follows = "substrate/nixpkgs";
    };
    crate2nix = {
      url = "github:nix-community/crate2nix";
    };
    flake-utils = {
      url = "github:numtide/flake-utils";
    };
    substrate = {
      url = "github:pleme-io/substrate";
    };
  };
  outputs = inputs @ { self, nixpkgs, crate2nix, flake-utils, substrate, ... }:
    (import "${substrate}/lib/rust-workspace-release-flake.nix" {
      inherit nixpkgs crate2nix flake-utils;
    }) {
      toolName = "masume";
      packageName = "masume";
      src = self;
      repo = "pleme-io/masume";
    };
}
