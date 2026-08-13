//! **M1 — SGR**, the family chosen because it was most likely to break M0's shape.
//!
//! M0 declared ten CSI cursor motions over a closed five-verb effect vocabulary
//! and matched a hand-written reference on 8,892 cases. Its own docs said what
//! that did *not* license: cursor movement is the easiest family — pure position
//! arithmetic, no colour, no modes — and a green run there says nothing about
//! SGR. This module is the experiment that tests it rather than assuming it.
//!
//! # The finding, stated before the code
//!
//! **M0's `Sequence` shape does NOT extend to SGR, and pretending otherwise
//! would have been the round-up.** A CSI cursor motion is *one sequence with
//! positional parameters*. `CSI m` is not: its parameter list is a **stream of
//! directives**, each of which may consume the parameters that follow it. Those
//! are different grammars, and a `Sequence { final_byte, params: [Param] }`
//! models only the first.
//!
//! What survives — and it is most of the value — is that the stream's
//! *alphabet* is a flat table. Roughly **90% of SGR is `code → effect`** with
//! ~40 entries, exactly the shape a catalog wants. The remaining 10% is the
//! extended-colour consumption rule, which is a **grammar, not a table**, and
//! stays hand-written code below. Saying "SGR is declarative" would be false;
//! saying "SGR's alphabet is declarative and its consumption rule is not" is
//! the honest split, and it is why [`SGR_CATALOG`] and
//! [`consume_extended_color`] are separate things in this file.
//!
//! # Why this family is worth the effort
//!
//! `tear-core`'s own `apply_sgr` carries a comment recording **two live
//! incidents**, both from this 10%:
//!
//! - `ESC[38:2::248:248:242m` — the ISO 8613-6 colon form carries a
//!   colour-space id in slot 2 which is almost always empty. Flattening colon
//!   and semicolon forms into one stream read that empty slot **as the red
//!   channel**: every channel shifted by one, and the real blue fell off the
//!   end and executed as an *attribute code*. When it happened to be `4`,
//!   UNDERLINE latched on for the rest of the session.
//! - `ESC[58;5;4m` — underline colour. 58 was dropped as unknown, so its
//!   parameters walked as attribute codes. Same stuck UNDERLINE, opposite
//!   direction.
//!
//! Both are the same defect: **a directive that is not consumed is executed.**
//! That is the bad state this module makes structural — [`SgrDirective`]
//! carries its own consumed-length, so "parsed but not advanced past" has no
//! representation.

use crate::{Color, Pen, palette};

// ─────────────────────────────────────────────────────────────────────────
// The declared alphabet
// ─────────────────────────────────────────────────────────────────────────

/// What a single SGR code does to the pen.
///
/// Closed, and closed is the point: an SGR code whose meaning is not one of
/// these has no declaration, so it lands in the catalog's absence rather than
/// in a silently-ignored `_ => {}` arm that looks identical to support.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PenEffect {
    Reset,
    Insert(Attr),
    Remove(Attr),
    /// Remove several at once — SGR 22 clears BOLD *and* DIM.
    RemoveBoth(Attr, Attr),
    /// A palette index, resolved against the pane's live palette.
    Foreground(PaletteSlot),
    Background(PaletteSlot),
    /// SGR 39 / 49 — the defaults, which are not palette entries.
    DefaultForeground,
    DefaultBackground,
}

/// Where a base-16 colour comes from. Resolved late, against the pane's own
/// palette, because OSC 4 can re-set it at runtime.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PaletteSlot(pub usize);

/// Cell attributes, as a bitset.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Attr(pub u16);

impl Attr {
    pub const NONE: Self = Self(0);
    pub const BOLD: Self = Self(1 << 0);
    pub const DIM: Self = Self(1 << 1);
    pub const ITALIC: Self = Self(1 << 2);
    pub const UNDERLINE: Self = Self(1 << 3);
    pub const BLINK: Self = Self(1 << 4);
    pub const INVERSE: Self = Self(1 << 5);
    pub const HIDDEN: Self = Self(1 << 6);
    pub const STRIKETHROUGH: Self = Self(1 << 7);

    #[must_use]
    pub const fn inserted(self, o: Self) -> Self {
        Self(self.0 | o.0)
    }
    #[must_use]
    pub const fn removed(self, o: Self) -> Self {
        Self(self.0 & !o.0)
    }
    #[must_use]
    pub const fn has(self, o: Self) -> bool {
        self.0 & o.0 != 0
    }
}

/// One declared SGR code.
#[derive(Debug, Clone, Copy)]
pub struct SgrCode {
    pub code: u16,
    pub name: &'static str,
    pub effect: PenEffect,
}

const fn c(code: u16, name: &'static str, effect: PenEffect) -> SgrCode {
    SgrCode { code, name, effect }
}

/// The flat 90%: `code → effect`, declared.
///
/// The 30–37 / 40–47 / 90–97 / 100–107 ranges are written out rather than
/// folded into an arithmetic arm. Sixteen extra rows buys a catalog whose
/// length equals the vocabulary's — which is the number a conformance matrix
/// is total over, and the number `--list` prints. A range arm covers eight
/// codes with one row and then quietly disagrees with both.
pub const SGR_CATALOG: &[SgrCode] = &[
    c(0, "reset", PenEffect::Reset),
    c(1, "bold", PenEffect::Insert(Attr::BOLD)),
    c(2, "dim", PenEffect::Insert(Attr::DIM)),
    c(3, "italic", PenEffect::Insert(Attr::ITALIC)),
    c(4, "underline", PenEffect::Insert(Attr::UNDERLINE)),
    c(5, "blink-slow", PenEffect::Insert(Attr::BLINK)),
    c(6, "blink-fast", PenEffect::Insert(Attr::BLINK)),
    c(7, "inverse", PenEffect::Insert(Attr::INVERSE)),
    c(8, "hidden", PenEffect::Insert(Attr::HIDDEN)),
    c(9, "strikethrough", PenEffect::Insert(Attr::STRIKETHROUGH)),
    // 21 is "doubly underlined" in ECMA-48 and "bold off" in much of the
    // wild. tear-core takes the latter and so does this — the catalog
    // reproduces the reference, divergences from the standard included.
    c(21, "bold-off", PenEffect::RemoveBoth(Attr::BOLD, Attr::DIM)),
    c(
        22,
        "normal-intensity",
        PenEffect::RemoveBoth(Attr::BOLD, Attr::DIM),
    ),
    c(23, "italic-off", PenEffect::Remove(Attr::ITALIC)),
    c(24, "underline-off", PenEffect::Remove(Attr::UNDERLINE)),
    c(25, "blink-off", PenEffect::Remove(Attr::BLINK)),
    c(27, "inverse-off", PenEffect::Remove(Attr::INVERSE)),
    c(28, "hidden-off", PenEffect::Remove(Attr::HIDDEN)),
    c(
        29,
        "strikethrough-off",
        PenEffect::Remove(Attr::STRIKETHROUGH),
    ),
    c(30, "fg-black", PenEffect::Foreground(PaletteSlot(0))),
    c(31, "fg-red", PenEffect::Foreground(PaletteSlot(1))),
    c(32, "fg-green", PenEffect::Foreground(PaletteSlot(2))),
    c(33, "fg-yellow", PenEffect::Foreground(PaletteSlot(3))),
    c(34, "fg-blue", PenEffect::Foreground(PaletteSlot(4))),
    c(35, "fg-magenta", PenEffect::Foreground(PaletteSlot(5))),
    c(36, "fg-cyan", PenEffect::Foreground(PaletteSlot(6))),
    c(37, "fg-white", PenEffect::Foreground(PaletteSlot(7))),
    c(39, "fg-default", PenEffect::DefaultForeground),
    c(40, "bg-black", PenEffect::Background(PaletteSlot(0))),
    c(41, "bg-red", PenEffect::Background(PaletteSlot(1))),
    c(42, "bg-green", PenEffect::Background(PaletteSlot(2))),
    c(43, "bg-yellow", PenEffect::Background(PaletteSlot(3))),
    c(44, "bg-blue", PenEffect::Background(PaletteSlot(4))),
    c(45, "bg-magenta", PenEffect::Background(PaletteSlot(5))),
    c(46, "bg-cyan", PenEffect::Background(PaletteSlot(6))),
    c(47, "bg-white", PenEffect::Background(PaletteSlot(7))),
    c(49, "bg-default", PenEffect::DefaultBackground),
    c(90, "fg-bright-black", PenEffect::Foreground(PaletteSlot(8))),
    c(91, "fg-bright-red", PenEffect::Foreground(PaletteSlot(9))),
    c(
        92,
        "fg-bright-green",
        PenEffect::Foreground(PaletteSlot(10)),
    ),
    c(
        93,
        "fg-bright-yellow",
        PenEffect::Foreground(PaletteSlot(11)),
    ),
    c(94, "fg-bright-blue", PenEffect::Foreground(PaletteSlot(12))),
    c(
        95,
        "fg-bright-magenta",
        PenEffect::Foreground(PaletteSlot(13)),
    ),
    c(96, "fg-bright-cyan", PenEffect::Foreground(PaletteSlot(14))),
    c(
        97,
        "fg-bright-white",
        PenEffect::Foreground(PaletteSlot(15)),
    ),
    c(
        100,
        "bg-bright-black",
        PenEffect::Background(PaletteSlot(8)),
    ),
    c(101, "bg-bright-red", PenEffect::Background(PaletteSlot(9))),
    c(
        102,
        "bg-bright-green",
        PenEffect::Background(PaletteSlot(10)),
    ),
    c(
        103,
        "bg-bright-yellow",
        PenEffect::Background(PaletteSlot(11)),
    ),
    c(
        104,
        "bg-bright-blue",
        PenEffect::Background(PaletteSlot(12)),
    ),
    c(
        105,
        "bg-bright-magenta",
        PenEffect::Background(PaletteSlot(13)),
    ),
    c(
        106,
        "bg-bright-cyan",
        PenEffect::Background(PaletteSlot(14)),
    ),
    c(
        107,
        "bg-bright-white",
        PenEffect::Background(PaletteSlot(15)),
    ),
];

#[must_use]
pub fn lookup_sgr(code: u16) -> Option<&'static SgrCode> {
    SGR_CATALOG.iter().find(|s| s.code == code)
}

pub fn apply_pen_effect(e: PenEffect, pal: &[Color; 16], pen: &mut Pen) {
    match e {
        PenEffect::Reset => *pen = Pen::default(),
        PenEffect::Insert(a) => pen.attrs = pen.attrs.inserted(a),
        PenEffect::Remove(a) => pen.attrs = pen.attrs.removed(a),
        PenEffect::RemoveBoth(a, b) => pen.attrs = pen.attrs.removed(a).removed(b),
        PenEffect::Foreground(PaletteSlot(i)) => pen.fg = pal[i],
        PenEffect::Background(PaletteSlot(i)) => pen.bg = pal[i],
        PenEffect::DefaultForeground => pen.fg = Color::WHITE,
        PenEffect::DefaultBackground => pen.bg = Color::BLACK,
    }
}

// ─────────────────────────────────────────────────────────────────────────
// The 10% that is a grammar, not a table
// ─────────────────────────────────────────────────────────────────────────

/// The outcome of reading one extended-colour directive.
///
/// **`consumed` is part of the value, not a separate return.** That is the
/// whole defence against the two incidents in this module's header: both were
/// "a directive was recognised but the parameters it owned were not skipped",
/// so the leftovers executed as attribute codes. A caller cannot obtain a
/// colour here without also obtaining how far to advance, and `consumed` is
/// never zero, so the loop cannot fail to progress either.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Consumed {
    pub color: Option<Color>,
    pub consumed: usize,
}

/// Semicolon form: `38;5;n` / `38;2;r;g;b`. `rest[0]` is the directive.
///
/// Returns `None` for 58 (underline colour, which we parse purely so it cannot
/// leak) and for malformed input — but *always* a non-zero `consumed`.
#[must_use]
pub fn consume_extended_color(rest: &[&[u16]], pal: &[Color; 16]) -> Consumed {
    let first = |i: usize| rest.get(i).and_then(|p| p.first().copied());
    match first(1) {
        Some(5) => match first(2) {
            Some(n) => Consumed {
                color: Some(ansi_256(n, pal)),
                consumed: 3,
            },
            None => Consumed {
                color: None,
                consumed: 2,
            },
        },
        Some(2) => match (first(2), first(3), first(4)) {
            (Some(r), Some(g), Some(b)) => Consumed {
                color: Some(Color::new(r as u8, g as u8, b as u8)),
                consumed: 5,
            },
            _ => Consumed {
                color: None,
                consumed: rest.len().min(5).max(1),
            },
        },
        _ => Consumed {
            color: None,
            consumed: 1,
        },
    }
}

/// Colon form: one parameter carrying its own sub-parameters, self-contained
/// by construction — which is why it needs no `consumed`.
///
/// The length test on the `2` arm is the fix for incident one: `38:2:cs:r:g:b`
/// has **six** slots and `38:2:r:g:b` has five. Choosing by length is what
/// keeps the channels aligned; reading positionally shifts every channel and
/// drops the real blue into the attribute stream.
#[must_use]
pub fn subparam_color(param: &[u16], pal: &[Color; 16]) -> Option<Color> {
    match param.get(1)? {
        5 => param.get(2).map(|&n| ansi_256(n, pal)),
        2 => match param.len() {
            n if n >= 6 => Some(Color::new(param[3] as u8, param[4] as u8, param[5] as u8)),
            5 => Some(Color::new(param[2] as u8, param[3] as u8, param[4] as u8)),
            _ => None,
        },
        _ => None,
    }
}

/// The xterm 256-colour cube.
#[must_use]
pub fn ansi_256(idx: u16, pal: &[Color; 16]) -> Color {
    match idx {
        0..=15 => pal[idx as usize],
        16..=231 => {
            let i = idx - 16;
            let to_byte = |v: u16| -> u8 { if v == 0 { 0 } else { (55 + 40 * v) as u8 } };
            Color::new(to_byte(i / 36), to_byte((i % 36) / 6), to_byte(i % 6))
        }
        232..=255 => {
            let v = (8 + 10 * (idx - 232)) as u8;
            Color::new(v, v, v)
        }
        _ => Color::WHITE,
    }
}

// ─────────────────────────────────────────────────────────────────────────
// The driver — alphabet + grammar composed
// ─────────────────────────────────────────────────────────────────────────

/// Apply a whole `CSI … m` to the pen.
pub fn apply_sgr(params: &[Vec<u16>], pen: &mut Pen) {
    let pal = palette();
    if params.is_empty() {
        *pen = Pen::default();
        return;
    }
    let items: Vec<&[u16]> = params.iter().map(Vec::as_slice).collect();
    let mut idx = 0;
    while idx < items.len() {
        let param = items[idx];
        let Some(&code) = param.first() else {
            idx += 1;
            continue;
        };

        // Colon form is self-contained; never let it reach the semicolon path.
        if param.len() > 1 {
            match code {
                // Styled underline: every style but `4:0` is "on"; `4:0` is the
                // modern spelling of SGR 24.
                4 => {
                    let on = param.get(1).copied().unwrap_or(0) != 0;
                    pen.attrs = if on {
                        pen.attrs.inserted(Attr::UNDERLINE)
                    } else {
                        pen.attrs.removed(Attr::UNDERLINE)
                    };
                }
                38 | 48 | 58 => {
                    let col = subparam_color(param, &pal);
                    match (code, col) {
                        (38, Some(c)) => pen.fg = c,
                        (48, Some(c)) => pen.bg = c,
                        // 58 is underline colour: parsed so it cannot leak,
                        // then dropped because there is nowhere to put it.
                        _ => {}
                    }
                }
                other => {
                    if let Some(s) = lookup_sgr(other) {
                        apply_pen_effect(s.effect, &pal, pen);
                    }
                }
            }
            idx += 1;
            continue;
        }

        if matches!(code, 38 | 48 | 58) {
            let got = consume_extended_color(&items[idx..], &pal);
            match (code, got.color) {
                (38, Some(c)) => pen.fg = c,
                (48, Some(c)) => pen.bg = c,
                _ => {}
            }
            idx += got.consumed;
            continue;
        }

        if let Some(s) = lookup_sgr(code) {
            apply_pen_effect(s.effect, &pal, pen);
        }
        idx += 1;
    }
}
