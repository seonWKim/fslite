# fslite — Project Instructions

## Source of truth

- `README.md` — architecture and design decisions.
- `MILESTONE.md` — implementation scope and sequence, phase 1 vs phase 2.

Read both before implementing anything. Don't implement ahead of the current
milestone, and don't add anything they don't describe without checking first.

## Rule 1 — Scope every change to one explicit purpose

- Keep changes small enough that a human reviewer isn't overwhelmed: one
  milestone, one bug fix, or one clearly stated task per change. Never bundle
  unrelated work into it.
- No speculative abstractions, no drive-by refactors, no "while I'm here"
  cleanup outside the stated purpose.
- If a task naturally spans multiple milestones, split it and confirm the
  split with the user first, rather than doing it all in one pass.

## Rule 2 — Confirm before diverging

- Before making a change, check it against `README.md`'s architecture and
  `MILESTONE.md`'s current scope.
- If a request conflicts with, isn't covered by, or reaches beyond what's
  documented there, stop and ask for clarification instead of guessing or
  silently reinterpreting it. Examples: a new component not in the
  architecture, a default that contradicts what's recorded, jumping ahead to
  a phase 2 item, or reintroducing something already explicitly rejected
  (e.g., a Turso Cloud dependency, a B+Tree index).
- If the user's answer changes a recorded decision, update `README.md` and/or
  `MILESTONE.md` to match before or alongside implementing — don't let the
  docs drift out of sync with the code.

## Rust tooling gates

Before considering any change done, all of the following must pass — treat a
failure as blocking, not a note to fix later:

- `cargo fmt --check` — formatting.
- `cargo clippy --all-targets -- -D warnings` — lint, warnings treated as errors.
- `cargo test` — existing tests must stay green; add tests for new behavior.
- `cargo build` — must compile without warnings.

## Commits & pushes

Never create a git commit or push in this repo. Leave the working tree with
staged or unstaged changes and let the human review the diff, write the
commit message, and push themselves.

