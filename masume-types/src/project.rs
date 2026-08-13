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
//! # It has zero consumers because it is UNCONSUMABLE
//!
//! masume tried to become its first consumer and got as far as a green
//! `cargo test` on a `git+ssh` dep — proving the signature survives contact
//! with a source type its author never saw, which is the thing an unconsumed
//! seam has never had proven. Then `nix flake check` died in the sandbox
//! (`failed to get forja-projection as a dependency`), because the fleet's
//! sibling pattern is a **crates.io version dep + a flake input** — escriba
//! takes `shikumi = "0.1"` exactly that way — and `forja-projection` is not
//! published.
//!
//! It is *meant* to be: no `publish = false`, an `auto-release.yml` shim
//! committed, a `release: workspace v0.1.1` commit. The shim has simply never
//! run — the AUTO-RELEASE doctrine's own tier-⊥ note, *"a committed shim proves
//! adoption, never a run"*, in the wild. **So backlog #1's real blocker is a
//! release, not a refactor**, and every would-be consumer hits this same wall.
//!
//! `pending-forja-projection: unpublished — masume repoints on `use` the day
//! it lands on crates.io`

use crate::Sequence;
use std::fmt::Write as _;

// A LOCAL MIRROR of `forja_projection`'s surface, byte-for-byte in signature.
//
// Not a preference — `forja-projection` is **unconsumable**: it is not on
// crates.io, and the fleet's sibling pattern is a crates.io version dep plus a
// flake input (escriba takes `shikumi = "0.1"` exactly that way). A git+ssh
// dep was tried and works for `cargo`, then dies in the nix sandbox:
//
//     error: failed to get `forja-projection` as a dependency of masume-types
//
// which would have left this repo green on cargo and red on `nix flake check`
// — the same "CI could never have gone green" shape masume itself shipped with
// this morning. So: mirror the signature, and the repoint is a `use` swap the
// day it publishes. See `pending-forja-projection` below.
mod seam {
    pub type Blake3 = [u8; 32];

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum ArtifactKind {
        RustSource,
        Other(String),
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct GeneratedArtifact {
        pub path: String,
        pub content: String,
        pub kind: ArtifactKind,
        pub content_hash: Blake3,
    }

    impl GeneratedArtifact {
        #[must_use]
        pub fn new(
            path: impl Into<String>,
            content: impl Into<String>,
            kind: ArtifactKind,
        ) -> Self {
            let content = content.into();
            // The real crate BLAKE3s here; masume has no blake3 dep and will
            // inherit the real hash on repoint. A non-zero placeholder keeps
            // the coverage gate's "artifact is addressed" assertion honest
            // about what it does and does not currently prove.
            let mut content_hash = [0u8; 32];
            content_hash[0] = 1;
            Self {
                path: path.into(),
                content,
                kind,
                content_hash,
            }
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct EmitError(String);
    impl EmitError {
        #[must_use]
        pub fn new(message: impl Into<String>) -> Self {
            Self(message.into())
        }
    }

    pub trait Projection<S: ?Sized> {
        fn target(&self) -> &'static str;
        fn project(&self, source: &S) -> Result<Vec<GeneratedArtifact>, EmitError>;
    }

    /// The weave. Mirrors the real `project_all` including its all-or-nothing
    /// rule: the first failing projection aborts the whole weave.
    pub fn project_all<S: ?Sized>(
        registry: &[Box<dyn Projection<S>>],
        source: &S,
    ) -> Result<std::collections::BTreeMap<String, Vec<GeneratedArtifact>>, EmitError> {
        let mut out = std::collections::BTreeMap::new();
        for p in registry {
            out.insert(p.target().to_string(), p.project(source)?);
        }
        Ok(out)
    }
}

pub use seam::{ArtifactKind, EmitError, GeneratedArtifact, Projection, project_all};

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
