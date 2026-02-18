# Snask v0.3.1 (Stable patch)

Snask v0.3.1 is a stable patch release on top of the v0.3.0 baseline, focused on Phase 1 (Foundation): stability, predictability, and better tooling UX.

## Highlights
- **`snask doctor`**: environment checks for toolchain, runtime artifacts, registry, and installed packages.
- **SPS improvements**:
  - clearer `snask.snif` parse errors (line/column + caret + “How to fix”),
  - `snask add/remove` preserves the `dependencies` block formatting when possible.
- **Docs**: stability policy, current status, and Phase 1 checklist.

## Stability model
Snask uses per-feature stability levels (Stable/Beta/Experimental). v0.3.1 is a stable patch release; experimental features remain experimental.

See:
- `docs/STATUS.md`
- `docs/STABILITY.md`

