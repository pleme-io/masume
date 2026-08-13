//! **M1 — SGR, differentially.**
//!
//! M0 tested the catalog shape on the easiest family and said so. This tests it
//! on the family most likely to break it: `CSI m` is not one sequence with
//! positional parameters, it is a **stream of directives** where a directive may
//! consume the parameters that follow it.
//!
//! The reference is a literal transcription of `tear-core/src/pane_grid.rs`'s
//! `apply_sgr` / `apply_sgr_subparams` / `parse_extended_color_params` /
//! `apply_sgr_code`, read 2026-08-13 — deliberately kept in the reference's own
//! shape, including its `21 => bold-off` divergence from ECMA-48, because the
//! question is migration safety, not standards purity.
//!
//! # The two incidents, as fixed cases
//!
//! `tear-core`'s comment records two production defects, both from the same
//! root: *a directive that is not consumed is executed*. Both are pinned below
//! by name, because an exhaustive sweep proves agreement while a named case
//! proves the specific historical failure cannot come back unnoticed.

use masume_types::sgr::{Attr, SGR_CATALOG, apply_sgr, lookup_sgr};
use masume_types::{Color, Pen, palette};

// ─────────────────────────────────────────────────────────────────────────
// The control: tear-core's apply_sgr, transcribed
// ─────────────────────────────────────────────────────────────────────────

mod reference {
    use masume_types::sgr::{Attr, ansi_256};
    use masume_types::{Color, Pen};

    fn sgr_reset(pen: &mut Pen) {
        pen.fg = Color::WHITE;
        pen.bg = Color::BLACK;
        pen.attrs = Attr::NONE;
    }

    fn apply_code(pen: &mut Pen, pal: &[Color; 16], p: u16) {
        match p {
            0 => sgr_reset(pen),
            1 => pen.attrs = pen.attrs.inserted(Attr::BOLD),
            2 => pen.attrs = pen.attrs.inserted(Attr::DIM),
            3 => pen.attrs = pen.attrs.inserted(Attr::ITALIC),
            4 => pen.attrs = pen.attrs.inserted(Attr::UNDERLINE),
            5 | 6 => pen.attrs = pen.attrs.inserted(Attr::BLINK),
            7 => pen.attrs = pen.attrs.inserted(Attr::INVERSE),
            8 => pen.attrs = pen.attrs.inserted(Attr::HIDDEN),
            9 => pen.attrs = pen.attrs.inserted(Attr::STRIKETHROUGH),
            21 | 22 => pen.attrs = pen.attrs.removed(Attr::BOLD).removed(Attr::DIM),
            23 => pen.attrs = pen.attrs.removed(Attr::ITALIC),
            24 => pen.attrs = pen.attrs.removed(Attr::UNDERLINE),
            25 => pen.attrs = pen.attrs.removed(Attr::BLINK),
            27 => pen.attrs = pen.attrs.removed(Attr::INVERSE),
            28 => pen.attrs = pen.attrs.removed(Attr::HIDDEN),
            29 => pen.attrs = pen.attrs.removed(Attr::STRIKETHROUGH),
            30..=37 => pen.fg = pal[(p - 30) as usize],
            39 => pen.fg = Color::WHITE,
            40..=47 => pen.bg = pal[(p - 40) as usize],
            49 => pen.bg = Color::BLACK,
            90..=97 => pen.fg = pal[8 + (p - 90) as usize],
            100..=107 => pen.bg = pal[8 + (p - 100) as usize],
            _ => {}
        }
    }

    fn subparams(pen: &mut Pen, pal: &[Color; 16], param: &[u16]) {
        match param[0] {
            4 => {
                if param[1] == 0 {
                    pen.attrs = pen.attrs.removed(Attr::UNDERLINE);
                } else {
                    pen.attrs = pen.attrs.inserted(Attr::UNDERLINE);
                }
            }
            code @ (38 | 48 | 58) => {
                let colour = match param[1] {
                    5 => param.get(2).map(|&n| ansi_256(n, pal)),
                    2 => match param.len() {
                        n if n >= 6 => {
                            Some(Color::new(param[3] as u8, param[4] as u8, param[5] as u8))
                        }
                        5 => Some(Color::new(param[2] as u8, param[3] as u8, param[4] as u8)),
                        _ => None,
                    },
                    _ => None,
                };
                match (code, colour) {
                    (38, Some(c)) => pen.fg = c,
                    (48, Some(c)) => pen.bg = c,
                    _ => {}
                }
            }
            other => apply_code(pen, pal, other),
        }
    }

    fn extended(rest: &[&[u16]], pal: &[Color; 16]) -> (Option<Color>, usize) {
        let first = |i: usize| rest.get(i).and_then(|p| p.first().copied());
        match first(1) {
            Some(5) => match first(2) {
                Some(n) => (Some(ansi_256(n, pal)), 3),
                None => (None, 2),
            },
            Some(2) => match (first(2), first(3), first(4)) {
                (Some(r), Some(g), Some(b)) => (Some(Color::new(r as u8, g as u8, b as u8)), 5),
                _ => (None, rest.len().min(5)),
            },
            _ => (None, 1),
        }
    }

    pub fn apply(params: &[Vec<u16>], pen: &mut Pen) {
        let pal = masume_types::palette();
        let items: Vec<&[u16]> = params.iter().map(Vec::as_slice).collect();
        if items.is_empty() {
            sgr_reset(pen);
            return;
        }
        let mut idx = 0;
        while idx < items.len() {
            let param = items[idx];
            let Some(&code) = param.first() else {
                idx += 1;
                continue;
            };
            if param.len() > 1 {
                subparams(pen, &pal, param);
                idx += 1;
                continue;
            }
            if matches!(code, 38 | 48 | 58) {
                let (colour, consumed) = extended(&items[idx..], &pal);
                match (code, colour) {
                    (38, Some(c)) => pen.fg = c,
                    (48, Some(c)) => pen.bg = c,
                    _ => {}
                }
                idx += consumed;
                continue;
            }
            apply_code(pen, &pal, code);
            idx += 1;
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────
// The corpus
// ─────────────────────────────────────────────────────────────────────────

fn one(n: u16) -> Vec<u16> {
    vec![n]
}

/// Every shape that has ever gone wrong here, plus the whole flat alphabet.
fn corpus() -> Vec<Vec<Vec<u16>>> {
    let mut out: Vec<Vec<Vec<u16>>> = Vec::new();

    // Empty = implicit reset.
    out.push(vec![]);

    // Every code 0..=120 alone — covers the declared alphabet AND the
    // undeclared gaps (10..20, 26, 38 bare, 50..89, 98, 99, 108+), which is
    // where "unknown, drop" must agree.
    for n in 0..=120u16 {
        out.push(vec![one(n)]);
    }

    // Attribute set/clear pairs, order-sensitive.
    for (on, off) in [
        (1u16, 22u16),
        (2, 22),
        (3, 23),
        (4, 24),
        (5, 25),
        (7, 27),
        (8, 28),
        (9, 29),
    ] {
        out.push(vec![one(on), one(off)]);
        out.push(vec![one(off), one(on)]);
        out.push(vec![one(on), one(0), one(off)]);
    }

    // Semicolon-form extended colour, well-formed and truncated at every point.
    for base in [38u16, 48, 58] {
        out.push(vec![one(base), one(5), one(9)]);
        out.push(vec![one(base), one(5), one(200)]);
        out.push(vec![one(base), one(5)]);
        out.push(vec![one(base)]);
        out.push(vec![one(base), one(2), one(1), one(2), one(3)]);
        out.push(vec![one(base), one(2), one(1), one(2)]);
        out.push(vec![one(base), one(2)]);
        out.push(vec![one(base), one(9)]);
        // Trailing directive AFTER a colour — the consumption bug's shape.
        out.push(vec![one(base), one(5), one(9), one(1)]);
        out.push(vec![one(base), one(2), one(1), one(2), one(3), one(4)]);
        out.push(vec![one(base), one(5), one(4)]);
    }

    // Colon form, both lengths, plus the empty colour-space slot.
    for base in [38u16, 48, 58] {
        out.push(vec![vec![base, 2, 0, 248, 248, 242]]);
        out.push(vec![vec![base, 2, 248, 248, 242]]);
        out.push(vec![vec![base, 5, 9]]);
        out.push(vec![vec![base, 5]]);
        out.push(vec![vec![base, 2]]);
        out.push(vec![vec![base, 9]]);
        out.push(vec![vec![base, 2, 0, 248, 248, 242], one(1)]);
    }

    // Styled underline, colon form.
    for n in 0..=5u16 {
        out.push(vec![vec![4, n]]);
        out.push(vec![vec![4, n], one(24)]);
    }

    // Long realistic runs.
    out.push(vec![one(0), one(1), one(4), one(31), one(47)]);
    out.push(vec![
        one(1),
        one(38),
        one(2),
        one(10),
        one(20),
        one(30),
        one(4),
    ]);
    out.push(vec![
        one(0),
        one(38),
        one(5),
        one(196),
        one(48),
        one(5),
        one(21),
        one(1),
    ]);
    out.push(vec![
        vec![38, 2, 0, 1, 2, 3],
        vec![48, 5, 9],
        one(1),
        one(22),
    ]);
    // An empty parameter in the middle — `ESC[1;;4m`.
    out.push(vec![one(1), vec![], one(4)]);

    out
}

// ─────────────────────────────────────────────────────────────────────────
// The gate
// ─────────────────────────────────────────────────────────────────────────

#[test]
fn m1_declared_sgr_matches_the_hand_written_reference() {
    let mut checked = 0usize;
    let mut diverged = Vec::new();

    // Every case is run from BOTH a fresh pen and a dirtied one: SGR is
    // stateful, and a bug that only shows when an attribute is already set
    // (the two real incidents were exactly that) is invisible from clean.
    let dirty = {
        let mut p = Pen::default();
        p.attrs = Attr::NONE
            .inserted(Attr::BOLD)
            .inserted(Attr::ITALIC)
            .inserted(Attr::INVERSE);
        p.fg = Color::new(1, 2, 3);
        p.bg = Color::new(4, 5, 6);
        p
    };

    for start in [Pen::default(), dirty] {
        for case in corpus() {
            let mut mine = start;
            apply_sgr(&case, &mut mine);

            let mut theirs = start;
            reference::apply(&case, &mut theirs);

            checked += 1;
            if mine != theirs {
                diverged.push(format!(
                    "CSI {case:?} m — declared {mine:?} vs reference {theirs:?}"
                ));
            }
        }
    }

    assert!(
        checked > 400,
        "corpus collapsed to {checked} cases — a differential that checks almost \
         nothing passes for the wrong reason",
    );
    assert!(
        diverged.is_empty(),
        "M1 RED — {} of {checked} diverge:\n  {}",
        diverged.len(),
        diverged
            .iter()
            .take(15)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n  "),
    );
    println!("M1 GREEN — {checked} SGR cases, 0 divergences");
}

// ── the two production incidents, pinned by name ─────────────────────────

/// Incident 1: the ISO 8613-6 colon form carries a colour-space id in slot 2,
/// almost always empty. Reading it positionally shifted every channel and
/// dropped the real blue into the attribute stream — where a `4` latched
/// UNDERLINE on for the rest of the session.
#[test]
fn incident_colon_form_channels_do_not_shift() {
    let mut pen = Pen::default();
    apply_sgr(&[vec![38, 2, 0, 248, 248, 242]], &mut pen);
    assert_eq!(
        pen.fg,
        Color::new(248, 248, 242),
        "the measured-wrong value was (0,248,248)"
    );
    assert!(
        !pen.attrs.has(Attr::UNDERLINE),
        "the dropped channel must not execute"
    );
}

/// Incident 2: SGR 58 is underline colour. Dropped as unknown, its parameters
/// walked as attribute codes — same stuck UNDERLINE, opposite direction.
#[test]
fn incident_sgr_58_is_consumed_not_executed() {
    let mut pen = Pen::default();
    apply_sgr(&[vec![58], vec![5], vec![4]], &mut pen);
    assert!(
        !pen.attrs.has(Attr::UNDERLINE),
        "58's parameters leaked and executed as attribute codes"
    );
}

/// The structural defence, stated as a property: a recognised extended-colour
/// directive always advances. `consumed == 0` would loop forever or re-execute.
#[test]
fn an_extended_color_directive_always_advances() {
    let pal = palette();
    for shape in [
        vec![vec![38u16]],
        vec![vec![38], vec![5]],
        vec![vec![38], vec![2]],
        vec![vec![38], vec![9]],
        vec![vec![48], vec![2], vec![1]],
    ] {
        let items: Vec<&[u16]> = shape.iter().map(Vec::as_slice).collect();
        let got = masume_types::sgr::consume_extended_color(&items, &pal);
        assert!(
            got.consumed >= 1,
            "consumed 0 for {shape:?} — the loop cannot progress"
        );
    }
}

/// The catalog is a set, not a list with duplicates, and its size is the
/// number a conformance matrix would be total over.
#[test]
fn the_sgr_catalog_has_no_duplicate_codes() {
    let mut codes: Vec<u16> = SGR_CATALOG.iter().map(|s| s.code).collect();
    let before = codes.len();
    codes.sort_unstable();
    codes.dedup();
    assert_eq!(before, codes.len(), "two SGR entries share a code");
    assert!(
        before >= 50,
        "expected the full flat alphabet, got {before}"
    );
}

/// Undeclared codes are ABSENT from the catalog, not present-and-ignored.
///
/// This is the seal the crate docs claim: "we support X" cannot be rounded up
/// from "we have an arm for X that does nothing", because there is no arm.
#[test]
fn undeclared_codes_are_absent_rather_than_silently_handled() {
    for gap in [10u16, 11, 20, 26, 50, 51, 73, 98, 99, 108, 200] {
        assert!(
            lookup_sgr(gap).is_none(),
            "SGR {gap} should not be declared"
        );
    }
    for known in [0u16, 1, 22, 31, 39, 47, 49, 90, 107] {
        assert!(
            lookup_sgr(known).is_some(),
            "SGR {known} should be declared"
        );
    }
}
