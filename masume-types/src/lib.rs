//! `masume-types` — the typed border.
//!
//! # Status: M0 ONLY. Ten sequences, one family.
//!
//! This crate holds the **frontier bisect** described in
//! [`theory/NATURALIZE-TERMINAL.md`][doc] §7 — not a terminal, not a catalog,
//! and emphatically not a claim that the design works at scale. It exists to
//! answer one question cheaply, before anyone writes ~1000 declarations:
//!
//! > Can a VT sequence be expressed as a **declaration** over a closed effect
//! > vocabulary, such that a dispatcher derived from that declaration
//! > reproduces a hand-written implementation **exactly**?
//!
//! [doc]: https://github.com/pleme-io/theory/blob/main/NATURALIZE-TERMINAL.md
//!
//! ## What is measured, and against what
//!
//! The CSI cursor-movement family — CUU, CUD, CUF, CUB, CNL, CPL, CHA, CUP,
//! HVP, VPA — declared in [`CATALOG`], interpreted by [`apply`], and
//! differentially tested against [`reference`], which is a **literal
//! transcription of `tear-core/src/pane_grid.rs`'s `csi_dispatch` arms** as
//! read on 2026-08-13. The reference is the control: it is what the fleet
//! ships today, clamping quirks and all.
//!
//! ## What this does NOT prove — read before quoting it
//!
//! - **Tokenizing is out of scope.** The differential feeds `(final_byte,
//!   params)` pairs directly. Turning bytes into those pairs is `vte`'s job
//!   (Paul Williams' published VT500 state diagram) and is a wire dependency,
//!   not part of the claim.
//! - **One family is not the vocabulary.** Cursor movement is the *easiest*
//!   family: pure position arithmetic, no buffer, no colour, no modes. SGR,
//!   the DEC private modes, DCS and OSC are all harder, and a green run here
//!   says nothing about them beyond "the shape survived first contact".
//! - **Expressiveness, not emission mechanics.** [`emit_dispatcher`] shows a
//!   catalog rendering to Rust source through a typed builder rather than
//!   `format!` of syntax (★★ TYPED EMISSION), but M0's verdict rests on the
//!   interpreter differential. Whether generated source is as *fast* as
//!   hand-written is unmeasured, and a terminal is a hot path.
//! - **The reference is one implementation, not the spec.** Agreement means
//!   "we reproduce tear-core", which is the migration-safety question. Where
//!   tear-core diverges from ECMA-48, this reproduces the divergence.

#![forbid(unsafe_code)]

// ─────────────────────────────────────────────────────────────────────────
// The declaration vocabulary
// ─────────────────────────────────────────────────────────────────────────

/// A CSI parameter, read positionally.
///
/// ECMA-48 lets a parameter be absent or zero and both mean "the default".
/// Carrying `min` on the declaration rather than in each effect is what lets
/// CUU's repeat-count and CUP's position share one resolution rule — the two
/// differ in what they *mean*, not in how the number is recovered.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Param {
    pub index: usize,
    pub min: u16,
}

impl Param {
    #[must_use]
    pub const fn at(index: usize) -> Self {
        Self { index, min: 1 }
    }

    /// Resolve against an actual parameter list.
    #[must_use]
    pub fn resolve(self, params: &[u16]) -> u16 {
        let raw = params.get(self.index).copied().unwrap_or(0);
        if raw < self.min { self.min } else { raw }
    }
}

/// Where an absolute coordinate comes from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pos {
    /// A **1-based** ECMA-48 coordinate parameter. Converted to a 0-based row
    /// or column by the interpreter — the off-by-one lives in exactly one
    /// place rather than in every arm that reads a position.
    OneBased(Param),
    /// Leave this axis where it is. CHA keeps the row; VPA keeps the column.
    Current,
}

/// A signed repeat count.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Delta {
    pub negative: bool,
    pub count: Param,
}

impl Delta {
    #[must_use]
    pub const fn forward(index: usize) -> Self {
        Self {
            negative: false,
            count: Param::at(index),
        }
    }
    #[must_use]
    pub const fn backward(index: usize) -> Self {
        Self {
            negative: true,
            count: Param::at(index),
        }
    }
}

/// **The closed effect vocabulary.**
///
/// This being closed is the seal. A declaration naming an effect the engine
/// cannot perform has no parse — which is where "we support sequence X" would
/// otherwise get rounded up from "we have an arm for X that ignores it".
///
/// Five variants cover all ten cursor motions. That ratio is the whole
/// argument in miniature: ten hand-written match arms, or ten declarations
/// over five verbs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Effect {
    /// Move vertically, clamped into the grid.
    MoveRows(Delta),
    /// Move horizontally, clamped into the grid.
    MoveCols(Delta),
    /// Set an absolute position, clamped high.
    SetPosition { row: Pos, col: Pos },
    /// Column 0.
    CarriageReturn,
    /// Run in order. `&'static` so the catalog stays a `const`.
    Then(&'static [Effect]),
}

/// One declared sequence.
#[derive(Debug, Clone, Copy)]
pub struct Sequence {
    /// The ECMA-48 mnemonic, e.g. `CUP`.
    pub name: &'static str,
    /// Human title, for the emitted docs.
    pub title: &'static str,
    /// The CSI final byte.
    pub final_byte: u8,
    pub effect: Effect,
}

// ─────────────────────────────────────────────────────────────────────────
// The catalog — ten declarations
// ─────────────────────────────────────────────────────────────────────────

const CNL_STEPS: &[Effect] = &[Effect::CarriageReturn, Effect::MoveRows(Delta::forward(0))];
const CPL_STEPS: &[Effect] = &[Effect::CarriageReturn, Effect::MoveRows(Delta::backward(0))];

/// The CSI cursor-movement family, declared.
///
/// `HVP` (`f`) and `CUP` (`H`) are separate entries with an identical effect
/// rather than one entry with two final bytes. They are two sequences that
/// happen to agree, and collapsing them would make the catalog's row count
/// stop matching the vocabulary's — which is the number the conformance matrix
/// is total over.
pub const CATALOG: &[Sequence] = &[
    Sequence {
        name: "CUU",
        title: "Cursor Up",
        final_byte: b'A',
        effect: Effect::MoveRows(Delta::backward(0)),
    },
    Sequence {
        name: "CUD",
        title: "Cursor Down",
        final_byte: b'B',
        effect: Effect::MoveRows(Delta::forward(0)),
    },
    Sequence {
        name: "CUF",
        title: "Cursor Forward",
        final_byte: b'C',
        effect: Effect::MoveCols(Delta::forward(0)),
    },
    Sequence {
        name: "CUB",
        title: "Cursor Back",
        final_byte: b'D',
        effect: Effect::MoveCols(Delta::backward(0)),
    },
    Sequence {
        name: "CNL",
        title: "Cursor Next Line",
        final_byte: b'E',
        effect: Effect::Then(CNL_STEPS),
    },
    Sequence {
        name: "CPL",
        title: "Cursor Previous Line",
        final_byte: b'F',
        effect: Effect::Then(CPL_STEPS),
    },
    Sequence {
        name: "CHA",
        title: "Cursor Horizontal Absolute",
        final_byte: b'G',
        effect: Effect::SetPosition {
            row: Pos::Current,
            col: Pos::OneBased(Param::at(0)),
        },
    },
    Sequence {
        name: "CUP",
        title: "Cursor Position",
        final_byte: b'H',
        effect: Effect::SetPosition {
            row: Pos::OneBased(Param::at(0)),
            col: Pos::OneBased(Param::at(1)),
        },
    },
    Sequence {
        name: "HVP",
        title: "Horizontal and Vertical Position",
        final_byte: b'f',
        effect: Effect::SetPosition {
            row: Pos::OneBased(Param::at(0)),
            col: Pos::OneBased(Param::at(1)),
        },
    },
    Sequence {
        name: "VPA",
        title: "Vertical Position Absolute",
        final_byte: b'd',
        effect: Effect::SetPosition {
            row: Pos::OneBased(Param::at(0)),
            col: Pos::Current,
        },
    },
];

/// Look a final byte up in the catalog.
#[must_use]
pub fn lookup(final_byte: u8) -> Option<&'static Sequence> {
    CATALOG.iter().find(|s| s.final_byte == final_byte)
}

// ─────────────────────────────────────────────────────────────────────────
// The interpreter
// ─────────────────────────────────────────────────────────────────────────

/// The slice of terminal state the cursor family touches. Nothing else.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cursor {
    pub row: usize,
    pub col: usize,
    pub rows: usize,
    pub cols: usize,
}

impl Cursor {
    #[must_use]
    pub const fn new(rows: usize, cols: usize) -> Self {
        Self {
            row: 0,
            col: 0,
            rows,
            cols,
        }
    }

    fn clamp_row(&self, r: isize) -> usize {
        let hi = self.rows.saturating_sub(1);
        if r < 0 { 0 } else { (r as usize).min(hi) }
    }

    fn clamp_col(&self, c: isize) -> usize {
        let hi = self.cols.saturating_sub(1);
        if c < 0 { 0 } else { (c as usize).min(hi) }
    }
}

fn delta_of(d: Delta, params: &[u16]) -> isize {
    let n = isize::from(d.count.resolve(params) as i32 as i16);
    let n = if n == 0 { 1 } else { n };
    if d.negative { -n } else { n }
}

fn pos_of(p: Pos, params: &[u16], current: usize) -> usize {
    match p {
        // The ONE place the 1-based→0-based conversion happens.
        Pos::OneBased(param) => param.resolve(params).saturating_sub(1) as usize,
        Pos::Current => current,
    }
}

/// Apply a declared effect to a cursor.
pub fn apply(effect: &Effect, params: &[u16], cur: &mut Cursor) {
    match effect {
        Effect::MoveRows(d) => {
            let target = cur.row as isize + delta_of(*d, params);
            cur.row = cur.clamp_row(target);
        }
        Effect::MoveCols(d) => {
            let target = cur.col as isize + delta_of(*d, params);
            cur.col = cur.clamp_col(target);
        }
        Effect::SetPosition { row, col } => {
            let r = pos_of(*row, params, cur.row);
            let c = pos_of(*col, params, cur.col);
            cur.row = cur.clamp_row(r as isize);
            cur.col = cur.clamp_col(c as isize);
        }
        Effect::CarriageReturn => cur.col = 0,
        Effect::Then(steps) => {
            for s in *steps {
                apply(s, params, cur);
            }
        }
    }
}

/// Dispatch a CSI final byte through the catalog.
///
/// Returns `false` for a final byte the catalog does not declare — the caller's
/// cue that this is outside M0's family, never a silent no-op.
pub fn dispatch(final_byte: u8, params: &[u16], cur: &mut Cursor) -> bool {
    match lookup(final_byte) {
        Some(seq) => {
            apply(&seq.effect, params, cur);
            true
        }
        None => false,
    }
}

// ─────────────────────────────────────────────────────────────────────────
// The reference — the CONTROL, transcribed from tear-core
// ─────────────────────────────────────────────────────────────────────────

/// A literal transcription of `tear-core/src/pane_grid.rs`'s `csi_dispatch`
/// cursor arms, read 2026-08-13.
///
/// This is deliberately ugly and deliberately hand-written: it is the thing the
/// catalog has to beat, and rewriting it "more cleanly" would weaken the
/// differential into a comparison of two things I wrote the same way.
///
/// Transcribed rules: `first = params[0] or 0`; `n = first.max(1)`;
/// `cursor_move_relative` clamps low at 0 and high at `len-1` on both axes;
/// `cursor_set` clamps high only; `E`/`F` do a carriage return first;
/// `H`/`f` default *each* of two params to 1 independently.
pub mod reference {
    use super::Cursor;

    fn move_relative(cur: &mut Cursor, drow: isize, dcol: isize) {
        let r = (cur.row as isize + drow).max(0) as usize;
        let c = (cur.col as isize + dcol).max(0) as usize;
        cur.row = r.min(cur.rows.saturating_sub(1));
        cur.col = c.min(cur.cols.saturating_sub(1));
    }

    fn set(cur: &mut Cursor, row: usize, col: usize) {
        cur.row = row.min(cur.rows.saturating_sub(1));
        cur.col = col.min(cur.cols.saturating_sub(1));
    }

    /// Returns false for a final byte outside the cursor family.
    #[must_use]
    pub fn dispatch(final_byte: u8, params: &[u16], cur: &mut Cursor) -> bool {
        let first = params.first().copied().unwrap_or(0);
        let n = isize::from(first.max(1) as i16);
        match final_byte {
            b'A' => move_relative(cur, -n, 0),
            b'B' => move_relative(cur, n, 0),
            b'C' => move_relative(cur, 0, n),
            b'D' => move_relative(cur, 0, -n),
            b'E' => {
                cur.col = 0;
                move_relative(cur, n, 0);
            }
            b'F' => {
                cur.col = 0;
                move_relative(cur, -n, 0);
            }
            b'G' => {
                let col = first.max(1) as usize - 1;
                let row = cur.row;
                set(cur, row, col);
            }
            b'H' | b'f' => {
                let row = params.first().copied().unwrap_or(1).max(1) as usize;
                let col = params.get(1).copied().unwrap_or(1).max(1) as usize;
                set(cur, row - 1, col - 1);
            }
            b'd' => {
                let row = first.max(1) as usize - 1;
                let col = cur.col;
                set(cur, row, col);
            }
            _ => return false,
        }
        true
    }
}

// ─────────────────────────────────────────────────────────────────────────
// Emission — the other half of the thesis
// ─────────────────────────────────────────────────────────────────────────

/// Render the catalog as Rust dispatcher source.
///
/// Built through `write!` on a `String` — a `Display`-family typed surface —
/// rather than `format!()` of syntax, per ★★ TYPED EMISSION. At catalog scale
/// this becomes a real AST + pretty-printer, the same discipline `NIX-AST` and
/// `GRAPHQL-AST` apply to their targets; at M0 it exists to show the catalog is
/// a *source of truth several artifacts derive from*, not just a lookup table.
#[must_use]
pub fn emit_dispatcher() -> String {
    use std::fmt::Write as _;
    let mut out = String::new();
    out.push_str("// GENERATED from masume_types::CATALOG. Do not edit.\n");
    out.push_str("match final_byte {\n");
    for s in CATALOG {
        let _ = writeln!(
            out,
            "    b'{}' => {{ /* {} — {} */ }}",
            s.final_byte as char, s.name, s.title
        );
    }
    out.push_str("    _ => return false,\n}\n");
    out
}

/// Render the catalog as a documentation table.
#[must_use]
pub fn emit_doc_table() -> String {
    use std::fmt::Write as _;
    let mut out = String::from("| Final | Mnemonic | Title |\n|---|---|---|\n");
    for s in CATALOG {
        let _ = writeln!(
            out,
            "| `{}` | {} | {} |",
            s.final_byte as char, s.name, s.title
        );
    }
    out
}

/// SGR — M1. See [`sgr`] for what that experiment found.
pub mod sgr;

/// An RGB colour.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub const WHITE: Self = Self {
        r: 255,
        g: 255,
        b: 255,
    };
    pub const BLACK: Self = Self { r: 0, g: 0, b: 0 };
    #[must_use]
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
}

/// The drawing pen: what the next printed cell inherits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pen {
    pub fg: Color,
    pub bg: Color,
    pub attrs: sgr::Attr,
}

impl Default for Pen {
    fn default() -> Self {
        Self {
            fg: Color::WHITE,
            bg: Color::BLACK,
            attrs: sgr::Attr::NONE,
        }
    }
}

/// A fixed 16-colour palette for the differential.
///
/// Deliberately NOT the fleet's real palette: M1 measures whether the declared
/// alphabet and the hand-written reference resolve the SAME slot, and any two
/// implementations agree trivially if every slot holds the same colour. Sixteen
/// distinguishable values make an off-by-one in a range arm visible.
#[must_use]
pub fn palette() -> [Color; 16] {
    let mut p = [Color::BLACK; 16];
    for (i, slot) in p.iter_mut().enumerate() {
        *slot = Color::new((i as u8) * 16 + 1, (i as u8) * 3 + 2, 255 - (i as u8) * 5);
    }
    p
}
