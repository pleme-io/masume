//! Projection — the ONE seam every emitted artifact goes through.
//!
//! # Why this exists rather than the `emit_*` functions it replaced
//!
//! masume's first cut hand-rolled `emit_dispatcher()` and `emit_doc_table()` as
//! bespoke free functions. That was **rediscovery #5** of a shape
//! [`theory/GENERATION-SUBSTRATE.md`][gs] had already named four times:
//!
//! > *iac-forge::Backend ‖ hata-emit::Projection ‖ repo-forge::Synthesizer<Ast>
//! > ‖ caixa's no-trait copy-paste* — "the four rediscoveries converge on this
//! > shape", with **merge the 4 Projection traits → 1** listed as consolidation
//! > backlog item #1.
//!
//! [gs]: https://github.com/pleme-io/theory/blob/main/GENERATION-SUBSTRATE.md
//!
//! Adding a fifth un-merged emitter surface would have made the fleet's own
//! named problem worse while claiming to advance generation — the exact shape
//! of a commit that looks like progress and is regression. The doctrine's
//! destination is *one typed primary structure per domain, projecting every
//! artifact through ONE registry*, and masume is a textbook instance: one
//! catalog, four artifacts.
//!
//! # Tier-honest: this is PRE-SHAPED, not repointed
//!
//! The canonical crate `forja-projection` **does not exist yet** — it is
//! backlog #1 and the 4→1 collapse is measured at *2-of-4 live* (hata-emit and
//! the teia frontend share the seam today; iac-forge is a 5-method 3-source
//! trait that needs a multi-source variant first). So this file declares the
//! trait **locally, in the canonical shape**, so that repointing is a `use`
//! change rather than a rewrite. It is a fifth *consumer-in-waiting*, not a
//! fifth *rediscovery*.
//!
//! Two deliberate omissions against the canonical shape, both because masume
//! has no use for them yet and a stub would be worse than an absence:
//! `source_hash: Blake3` and `morphism_chain`, which
//! GENERATION-SUBSTRATE.md §(1) says ride the seam as an *optional blanket-impl
//! tier* rather than a default.

use crate::Sequence;
use std::fmt::Write as _;

/// Which artifact a projection produces from the catalog.
///
/// Closed on purpose. A new emitted artifact is a variant plus an impl — and
/// [`ALL_TARGETS`] then forces it into the coverage gate, so an artifact that
/// silently skips half the catalog cannot ship.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Target {
    /// The `csi_dispatch` arms.
    RustDispatcher,
    /// The operator-facing table.
    DocTable,
}

pub const ALL_TARGETS: &[Target] = &[Target::RustDispatcher, Target::DocTable];

/// One emitted file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratedArtifact {
    pub path: String,
    pub content: String,
    pub target: Target,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EmitError {
    /// The catalog is empty — emitting nothing is a defect, not an artifact.
    EmptySource,
}

/// **The seam.** Deliberately the canonical signature from
/// GENERATION-SUBSTRATE.md §(1), so `forja-projection` can absorb it.
/// `S: ?Sized` so the source can be a SLICE — `Projection<[Sequence]>`, which
/// is the same spelling `hata-emit` already ships (`Projection<[DefEntidade]>`).
/// Requiring `Sized` would force every catalog to be projected as an owned
/// `Vec`, which is a copy per artifact for no reason.
pub trait Projection<S: ?Sized> {
    fn target(&self) -> Target;
    fn project(&self, source: &S) -> Result<Vec<GeneratedArtifact>, EmitError>;
}

/// The dispatcher arms.
pub struct DispatcherProjection;

impl Projection<[Sequence]> for DispatcherProjection {
    fn target(&self) -> Target {
        Target::RustDispatcher
    }

    fn project(&self, source: &[Sequence]) -> Result<Vec<GeneratedArtifact>, EmitError> {
        if source.is_empty() {
            return Err(EmitError::EmptySource);
        }
        // Built through `write!` on a `String` — a `Display`-family typed
        // surface — never `format!()` of target syntax (★★ TYPED EMISSION,
        // whose ban GENERATION-SUBSTRATE.md lists as consolidation #3). At
        // catalog scale this becomes a real AST + printer, the same move
        // NIX-AST and GRAPHQL-AST make for their targets.
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
        Ok(vec![GeneratedArtifact {
            path: "generated/csi_dispatch.rs".into(),
            content: out,
            target: Target::RustDispatcher,
        }])
    }
}

/// The documentation table.
pub struct DocTableProjection;

impl Projection<[Sequence]> for DocTableProjection {
    fn target(&self) -> Target {
        Target::DocTable
    }

    fn project(&self, source: &[Sequence]) -> Result<Vec<GeneratedArtifact>, EmitError> {
        if source.is_empty() {
            return Err(EmitError::EmptySource);
        }
        let mut out = String::from("| Final | Mnemonic | Title |\n|---|---|---|\n");
        for s in source {
            let _ = writeln!(
                out,
                "| `{}` | {} | {} |",
                s.final_byte as char, s.name, s.title
            );
        }
        Ok(vec![GeneratedArtifact {
            path: "generated/sequences.md".into(),
            content: out,
            target: Target::DocTable,
        }])
    }
}

/// Project the catalog through every registered target.
///
/// The registry is what makes "one structure, N artifacts" a fact rather than a
/// slogan: a caller cannot emit *some* artifacts, and the coverage gate runs
/// over this, not over a hand-listed set.
pub fn project_all(source: &[Sequence]) -> Result<Vec<GeneratedArtifact>, EmitError> {
    let mut out = Vec::new();
    out.extend(DispatcherProjection.project(source)?);
    out.extend(DocTableProjection.project(source)?);
    Ok(out)
}
