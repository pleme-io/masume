//! Projection — masume's emitted artifacts, on the fleet's ONE seam.
//!
//! # This file used to be a rediscovery. Twice.
//!
//! masume's first cut hand-rolled `emit_dispatcher()` / `emit_doc_table()` as
//! free functions. Told to standardise, the second cut declared a *local*
//! `Projection<S>` trait "pre-shaped for the repoint" — on the belief, stated
//! in its own doc comment, that `forja-projection` "does not exist yet".
//!
//! **It exists.** `pleme-io/forja-projection` v0.1.1, carrying the trait, a
//! content-addressed [`GeneratedArtifact`] (BLAKE3 on construction), a typed
//! [`EmitError`], an [`ArtifactKind`] vocabulary, **and** a
//! [`forja_projection::project_all`] registry-weave — which the local version
//! had *also* independently re-rolled as a free function.
//!
//! So the count was: the fleet rediscovered this shape four times, the doctrine
//! named the consolidation, the crate was then actually extracted — and masume
//! still rediscovered it twice more, the second time while explicitly citing
//! the document that says the crate is backlog #1. **The doctrine was read and
//! the repo was not.** A doctrine doc records intent at the time of writing; a
//! `find . -maxdepth 1 -type d -name 'forja*'` records reality, costs one
//! command, and is what actually settles whether a primitive exists.
//!
//! # masume is its FIRST consumer — as of 2026-08-13
//!
//! The crate had zero consumers, and not for want of anyone trying: it was
//! **private**, so substrate's `cargo-auto-release.yml` — which gates `ship`
//! on `!github.event.repository.private` — skipped publishing on every one of
//! its three green runs. Unpublished ⇒ unconsumable (the fleet's sibling
//! pattern is a crates.io version dep + a flake input) ⇒ no consumer ⇒ which
//! was, circularly, `org.yaml`'s stated reason for keeping it private:
//! *"nothing open depends on it"*.
//!
//! masume going public broke the loop. The repo was opened through the IaC
//! path (`pangea-architectures@a94a3f3`, approved as
//! `akeylesslabs/k8s@bdc8b8b`), and the workflow's *"Registry catch-up (seals
//! the skipped-ship green)"* job — which exists for exactly this and had never
//! once run — published `0.1.1` in eleven seconds.
//!
//! So this file no longer mirrors the seam. It **is** the seam, and masume
//! gets real BLAKE3 content-addressing instead of the placeholder the mirror
//! carried. The only accommodation is cosmetic: `target()` returns
//! `&'static str`, so the coverage gate keys on strings.

use crate::Sequence;
use std::fmt::Write as _;

pub use forja_projection::{ArtifactKind, EmitError, GeneratedArtifact, Projection, project_all};

/// Target names — the registry keys. `&'static str` because that is the seam's
/// own choice; a local enum here would be a sixth private vocabulary.
pub const TARGET_DISPATCHER: &str = "masume::csi_dispatch";
pub const TARGET_DOC_TABLE: &str = "masume::sequence_docs";

/// Every registered target, so the coverage gate is total over the REGISTRY
/// rather than over a hand-listed pair.
pub const ALL_TARGETS: &[&str] = &[TARGET_DISPATCHER, TARGET_DOC_TABLE];

/// The dispatcher arms.
pub struct DispatcherProjection;

impl Projection<[Sequence]> for DispatcherProjection {
    fn target(&self) -> &'static str {
        TARGET_DISPATCHER
    }

    fn project(&self, source: &[Sequence]) -> Result<Vec<GeneratedArtifact>, EmitError> {
        if source.is_empty() {
            return Err(EmitError::new(
                "empty catalog: emitting a dispatcher with no arms is a defect, not an artifact",
            ));
        }
        // `write!` on a `String` — a `Display`-family typed surface — never
        // `format!()` of target syntax (★★ TYPED EMISSION, which the same
        // doctrine lists as consolidation #3). At catalog scale this becomes a
        // real AST + printer, as NIX-AST and GRAPHQL-AST do for their targets.
        let mut out = String::new();
        out.push_str("// GENERATED from masume_types::CATALOG. Do not edit.\n");
        out.push_str("match final_byte {\n");
        for s in source {
            let _ = writeln!(
                out,
                "    b'{}' => {{ /* {} — {} */ }}",
                s.final_byte as char, s.name, s.title
            );
        }
        out.push_str("    _ => return false,\n}\n");
        Ok(vec![GeneratedArtifact::new(
            "generated/csi_dispatch.rs",
            out,
            ArtifactKind::RustSource,
        )])
    }
}

/// The documentation table.
pub struct DocTableProjection;

impl Projection<[Sequence]> for DocTableProjection {
    fn target(&self) -> &'static str {
        TARGET_DOC_TABLE
    }

    fn project(&self, source: &[Sequence]) -> Result<Vec<GeneratedArtifact>, EmitError> {
        if source.is_empty() {
            return Err(EmitError::new(
                "empty catalog: a doc table with no rows reads as 'nothing is supported'",
            ));
        }
        let mut out = String::from("| Final | Mnemonic | Title |\n|---|---|---|\n");
        for s in source {
            let _ = writeln!(
                out,
                "| `{}` | {} | {} |",
                s.final_byte as char, s.name, s.title
            );
        }
        Ok(vec![GeneratedArtifact::new(
            "generated/sequences.md",
            out,
            // Markdown has no `ArtifactKind` variant; `Other` names it rather
            // than mislabelling it as one of the nine that do exist.
            ArtifactKind::Other("markdown".into()),
        )])
    }
}

/// masume's projection registry.
#[must_use]
pub fn registry() -> Vec<Box<dyn Projection<[Sequence]>>> {
    vec![Box::new(DispatcherProjection), Box::new(DocTableProjection)]
}
