//! `masume` — the CLI.
//!
//! Nothing is implemented. This binary exists so the declared workspace is a
//! real, building workspace rather than a manifest pointing at absent members,
//! and it reports its own status instead of pretending to a surface it does not
//! have. See `masume-types` for why M0 gates the first line of real code.

#![forbid(unsafe_code)]

fn main() {
    // Printed rather than `unimplemented!()`: a panic would say "broken", and
    // this is not broken — it is a declared shape whose contents are gated on a
    // measurement that has not run. The distinction is the whole point of the
    // tier ledger in the theory doc.
    print!(
        "\
masume (升目) — the ruled squares of a page

STATUS: DESIGN. Zero implementation.

A terminal substrate whose VT dispatch table, conformance matrix, terminfo
entry and documentation are all EMITTED from one typed sequence catalog rather
than hand-written — so two faces cannot disagree about a table neither of them
writes, and terminfo cannot drift from the implementation.

Naturalizes the ESSENCE of Ghostty (the core is a library), kitty (extend the
spec and publish it), Alacritty (a separable tokenizer), contour (conformance
as a product property) — rather than vendoring any of them.

  theory : https://github.com/pleme-io/theory/blob/main/NATURALIZE-TERMINAL.md
  naming : masume ratified 2026-08-13, opens The Page 頁

NEXT — M0, a frontier bisect and not a build:
  declare ~10 CSI cursor-movement sequences, emit the dispatcher arms, and run
  them against the two hand-written implementations that already exist
  (tear-core pane_grid.rs, 3253 lines / 95 tests; mado terminal.rs, 13453
  lines / 312 tests — both measured 2026-08-13). Green means the rest is
  declaration work. Red means we learn why before a catalog exists to be wrong.
"
    );
}
