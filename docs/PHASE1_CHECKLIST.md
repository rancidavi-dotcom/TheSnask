# Phase 1 Checklist (v0.3 → v0.4)

Goal: **stability + predictability + better error UX**.

## 1) Core stability policy
- Define what “Stable / Beta / Experimental” means in Snask.
- Define compatibility rules (what can break between patch/minor versions).
- Define what is part of the “core” vs “libraries”.
- Document the import-only native policy (reserved native symbols).

## 2) Diagnostics baseline (compiler UX)
Baseline expectations:
- Every error includes **file + line + column**.
- Every common error includes a **How to fix** hint.
- Messages are **English-only** in the compiler/runtime.

Concrete tasks:
- Standardize diagnostics formatting across parser/semantic/build/link steps.
- Add “most likely fix” hints for:
  - missing `;`
  - indentation block issues
  - calling import-only native functions
  - missing `class main` / missing `start`
  - unknown function / misspelled name (“did you mean”)

## 3) SPS + SNIF reliability
- Ensure `snask init` always generates a valid `snask.snif`.
- Ensure `snask add/remove` preserves formatting and never corrupts the manifest.
- Ensure errors clearly say whether the failure is:
  - manifest parse
  - registry fetch
  - dependency integrity (sha256)
  - compile vs link

