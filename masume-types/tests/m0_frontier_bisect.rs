//! **M0 — the frontier bisect.**
//!
//! The measurement `theory/NATURALIZE-TERMINAL.md` §7 gates the whole design
//! on. One question, asked as cheaply as it can be asked:
//!
//! > Can a VT sequence be expressed as a DECLARATION over a closed effect
//! > vocabulary, such that a dispatcher derived from it reproduces a
//! > hand-written implementation exactly?
//!
//! Method: an exhaustive differential of [`masume_types::dispatch`] (derived
//! from `CATALOG`) against [`masume_types::reference::dispatch`] (a literal
//! transcription of `tear-core/src/pane_grid.rs`, read 2026-08-13) over every
//! combination of final byte × parameter shape × starting cursor × grid size.
//!
//! A green run means the remaining ~1000 sequences are declaration work *for
//! this family's shape*. It does not mean SGR or the DEC modes will fall out
//! the same way — see the crate docs for what is deliberately not claimed.
//!
//! # Red run — 2026-08-13
//!
//! A gate that has never gone red is a gate nobody has shown can fail.
//! Deleting the `.saturating_sub(1)` from `pos_of` — a one-token off-by-one,
//! and the single likeliest real error in this family — produced:
//!
//! ```text
//! M0 RED — 2112 of 8892 cases diverge
//! test m0_reproduces_tear_cores_own_asserted_vector ... FAILED
//! test m0_declared_dispatch_matches_the_hand_written_reference ... FAILED
//! ```
//!
//! Both the exhaustive differential AND the transcription check caught it
//! independently, which is what those two tests exist to do separately. The
//! break was reverted and the suite verified green again.

use masume_types::project::{ALL_TARGETS, project_all, registry};
use masume_types::{CATALOG, Cursor, dispatch, reference};

/// Every final byte in the family, plus two that are NOT in it — the negative
/// control. Without those, a `dispatch` that returned `true` for everything
/// would pass this file.
const FINALS: &[u8] = &[
    b'A', b'B', b'C', b'D', b'E', b'F', b'G', b'H', b'f', b'd', // in family
    b'J', b'm', // NOT in family
];

/// Parameter shapes that matter. `[]` and `[0]` are the two spellings of
/// "default", and they are where an off-by-one hides; 1 is the identity;
/// 99 overruns every grid here, which is what exercises the clamps.
fn param_shapes() -> Vec<Vec<u16>> {
    vec![
        vec![],
        vec![0],
        vec![1],
        vec![2],
        vec![5],
        vec![99],
        vec![0, 0],
        vec![1, 1],
        vec![3, 5],
        vec![5, 3],
        vec![99, 99],
        vec![0, 7],
        vec![7, 0],
    ]
}

fn grids() -> Vec<(usize, usize)> {
    // Including 1x1 deliberately: `len - 1` underflows there if anyone reaches
    // for plain subtraction instead of `saturating_sub`.
    vec![(1, 1), (5, 10), (10, 5), (24, 80)]
}

#[test]
fn m0_declared_dispatch_matches_the_hand_written_reference() {
    let mut checked = 0usize;
    let mut divergences = Vec::new();

    for &(rows, cols) in &grids() {
        for start_row in [0usize, 1, rows / 2, rows.saturating_sub(1)] {
            for start_col in [0usize, 1, cols / 2, cols.saturating_sub(1)] {
                if start_row >= rows || start_col >= cols {
                    continue;
                }
                for shape in &param_shapes() {
                    for &fb in FINALS {
                        let base = Cursor {
                            row: start_row,
                            col: start_col,
                            rows,
                            cols,
                        };

                        let mut mine = base;
                        let mine_handled = dispatch(fb, shape, &mut mine);

                        let mut theirs = base;
                        let theirs_handled = reference::dispatch(fb, shape, &mut theirs);

                        checked += 1;

                        if mine_handled != theirs_handled || mine != theirs {
                            divergences.push(format!(
                                "grid {rows}x{cols} start ({start_row},{start_col}) \
                                 CSI {shape:?} '{}' — declared: handled={mine_handled} \
                                 ({},{})  reference: handled={theirs_handled} ({},{})",
                                fb as char, mine.row, mine.col, theirs.row, theirs.col,
                            ));
                        }
                    }
                }
            }
        }
    }

    assert!(
        checked > 2000,
        "the corpus collapsed to {checked} cases — a differential that checks \
         almost nothing passes for the wrong reason",
    );

    assert!(
        divergences.is_empty(),
        "M0 RED — {} of {checked} cases diverge:\n  {}",
        divergences.len(),
        divergences
            .iter()
            .take(15)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n  "),
    );

    println!("M0 GREEN — {checked} cases, 0 divergences");
}

/// tear-core's own asserted expectation, lifted verbatim from
/// `pane_grid.rs::cursor_move_csi_cup`: a 10x5 grid, `ESC [ 3 ; 5 H`, cursor
/// lands at row 2 col 4.
///
/// The exhaustive differential above proves *agreement with the reference*.
/// This proves the reference itself was transcribed correctly — without it, a
/// transcription error would make both sides wrong together and the test green.
#[test]
fn m0_reproduces_tear_cores_own_asserted_vector() {
    // PaneGrid::new(10, 5) is (cols, rows) in tear-core's constructor.
    let mut cur = Cursor::new(5, 10);
    assert!(dispatch(b'H', &[3, 5], &mut cur));
    assert_eq!(cur.row, 2, "row");
    assert_eq!(cur.col, 4, "col");
}

/// The catalog covers exactly the family it claims to, no more.
#[test]
fn m0_catalog_declares_ten_sequences_with_unique_finals() {
    assert_eq!(CATALOG.len(), 10, "the declared family size");
    let mut finals: Vec<u8> = CATALOG.iter().map(|s| s.final_byte).collect();
    finals.sort_unstable();
    let before = finals.len();
    finals.dedup();
    assert_eq!(before, finals.len(), "two sequences share a final byte");
}

/// The emission half: several artifacts derive from the one catalog.
///
/// Asserted by SET rather than by golden text — a golden file would pin the
/// formatting, which is not the claim. The claim is that every declared
/// sequence reaches every emitted artifact, so adding a row to the catalog
/// cannot leave one of them behind.
#[test]
fn m0_every_declared_sequence_reaches_every_emitted_artifact() {
    // Through forja-projection's OWN registry weave — the seam's `project_all`,
    // not a local re-roll of it. masume is that crate's first consumer.
    let by_target = project_all(&registry(), CATALOG).expect("catalog projects");

    // Total over the REGISTRY, not a hand-listed pair: a target with no impl,
    // or an impl that silently skips sequences, fails here.
    assert_eq!(
        by_target.len(),
        ALL_TARGETS.len(),
        "every registered target must appear; got {:?}",
        by_target.keys().collect::<Vec<_>>(),
    );
    for t in ALL_TARGETS {
        let arts = by_target
            .get(*t)
            .unwrap_or_else(|| panic!("no artifact for target {t}"));
        for a in arts {
            for s in CATALOG {
                assert!(
                    a.content.contains(s.name),
                    "{} missing from {t} ({})",
                    s.name,
                    a.path
                );
            }
            // The seam BLAKE3s on construction — real provenance since the
            // 2026-08-13 repoint off the local mirror, whose placeholder hash
            // could only support a "not all zero" check.
            assert_ne!(a.content_hash, [0u8; 32], "{t}: unhashed artifact");
        }
    }

    // An empty catalog is a typed error, not a valid-looking empty artifact.
    assert!(
        project_all(&registry(), &[][..]).is_err(),
        "an empty source must not project"
    );

    // The content-address is CONTENT-derived, not a per-artifact constant.
    // Only assertable since the repoint: the local mirror's placeholder was
    // the same bytes for every artifact, so this would have failed against it.
    let hashes: Vec<_> = by_target
        .values()
        .flatten()
        .map(|a| a.content_hash)
        .collect();
    assert_eq!(hashes.len(), 2, "two artifacts expected");
    assert_ne!(
        hashes[0], hashes[1],
        "two different artifacts share a content hash — the address is not content-derived"
    );

    // ...and it is STABLE: projecting twice addresses identically, which is
    // what a freshness gate would later stand on.
    let again = project_all(&registry(), CATALOG).expect("catalog re-projects");
    for t in ALL_TARGETS {
        assert_eq!(
            by_target[*t][0].content_hash, again[*t][0].content_hash,
            "{t}: content address is not stable across projections"
        );
    }
}
