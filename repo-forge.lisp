(defrepo masume
  :description "Masume (升目) — the ruled squares of a page. A terminal substrate whose VT dispatch table, conformance matrix, terminfo entry and docs are all EMITTED from one typed sequence catalog rather than hand-written, so two faces cannot disagree about a table neither of them writes. Naturalizes the essence of Ghostty / kitty / WezTerm / Alacritty / contour rather than vendoring any of them. Theory — theory/NATURALIZE-TERMINAL.md. Name ratified 2026-08-13, opens The Page 頁."
  :kind :rust-workspace-tool
  :visibility :public
  :binary "masume"
  :package-name "masume"
  :workspace-members ("masume-types" "masume")
  :ci
    (:systems       ("aarch64-darwin" "x86_64-darwin" "x86_64-linux" "aarch64-linux")
     :test-systems  ("aarch64-darwin")
     :build-images  #f))
