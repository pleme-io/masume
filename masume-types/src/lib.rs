//! `masume-types` — the typed border.
//!
//! # Status: EMPTY, deliberately.
//!
//! This crate is a declared shape with no contents yet, and that is the honest
//! state rather than an oversight. The design lives in
//! [`theory/NATURALIZE-TERMINAL.md`][doc] and is **DESIGN, zero code**; its own
//! §9 lists seven things it does not verify. Writing types here before **M0**
//! runs would be committing to a shape the measurement has not endorsed.
//!
//! [doc]: https://github.com/pleme-io/theory/blob/main/NATURALIZE-TERMINAL.md
//!
//! ## What M0 is, and why it gates this file
//!
//! M0 is a **frontier bisect**, not a build: declare ~10 CSI cursor-movement
//! sequences (CUP, CUU, CUD, CUF, CUB, CNL, CPL, CHA, VPA, HVP), emit the
//! dispatcher arms from those declarations, and run the result against the two
//! hand-written implementations that already exist —
//! `tear-core/src/pane_grid.rs` (3,253 lines, 95 tests + 4 proptests) and
//! `mado/src/terminal.rs` (13,453 lines, 312 tests), both measured 2026-08-13.
//!
//! Green means the remaining ~1000 sequences are declaration work and this
//! crate gets its border. Red means we learn why **before** a catalog exists to
//! be wrong. Either outcome is cheap; guessing is not.
//!
//! ## What will live here when it does
//!
//! The border half of the TYPED-SPEC + INTERPRETER triplet — a `Sequence`
//! declaration (kind, final byte, intermediates, params + defaults, effect,
//! origin, terminfo name), a **closed** `Effect` enum, and the cell/grid types.
//! `Effect` being closed is the seal: a declaration naming an effect the engine
//! cannot perform has no parse, which is where "we support sequence X" would
//! otherwise get rounded up from "we have an arm for X that ignores it".
//!
//! The interpreter and the `(defmasume …)` authoring surface are the other two
//! legs; the catalog form's own name is still unminted
//! (`pending-naming: masume-catalog-form` — four candidates died on the corpus
//! sweep, recorded in `theory/NAMING.md`).

#![forbid(unsafe_code)]
