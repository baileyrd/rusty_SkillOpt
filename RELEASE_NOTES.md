# Release Notes

Tracks notable changes to this repo, reverse chronological. As of PR #1, every
change lands through a PR against `main` (merge commit, green CI required);
entries from before that point predate the PR workflow and are keyed by
commit instead.

---

## Add smoke_claude_hard_bigtrain.yaml: does a bigger training pool find the gap?
**2026-07-23** · [PR #4](https://github.com/baileyrd/rusty_SkillOpt/pull/4)

- **Added:** `configs/smoke_claude_hard_bigtrain.yaml` — same difficulty knobs
  and val/test size as `smoke_claude_hard_bigval.yaml`, but `train_size`
  bumped from 8 to 32 (`epochs` dropped 2 -> 1 to avoid compounding the size
  increase with a second epoch's calls).
- Run result: **1/8 steps accepted, val 0.938 -> 1.0, test 1.0** (up from
  0.875 in the 8-example version). Confirms the earlier diagnosis: a bigger,
  more representative training pool surfaced a real failure and the loop
  produced a genuinely generalizing fix — the accepted edit came from a
  batch that itself scored a perfect 1.0 in training, yet still measurably
  improved val, and test went from 14/16 to a clean 16/16 afterward. The
  accepted skill adds explicit sequential step-by-step + double-check
  guidance, exactly what a 4-op chained problem needs.
- Also confirms an edge case: once at ceiling, the optimizer correctly
  proposed *zero-op* edits for 4 consecutive batches ("already at ceiling,
  no changes") instead of inventing busywork, and the engine correctly
  treats an empty edit as a rejection rather than erroring.

## Add smoke_claude_hard_bigval.yaml: bigger val/test to cut measurement noise
**2026-07-22** · [PR #3](https://github.com/baileyrd/rusty_SkillOpt/pull/3)

- **Added:** `configs/smoke_claude_hard_bigval.yaml` — same difficulty knobs
  as `smoke_claude_hard.yaml` (multi-step chaining, heavier distractors) but
  val/test bumped from 4 to 16 examples each. Running the 4-example version 6
  times showed val flipping between 0.75 and 1.0 run to run — at that size a
  single wrong answer swings the score by 0.25, making "does it top out" hard
  to distinguish from noise.
- Run result: val 1.0 -> 1.0 (0/4 accepted, all 16 val examples correct every
  step), but **test score 0.875** (2/16 wrong). Verified both failures
  (`test-36`, `test-38`) are legitimate 4-op chained problems with correct
  expected values, not another generator bug. Real finding: with only 8
  training examples, the loop never happened to see a chain hard enough to
  trigger a training failure and give the optimizer something to react to,
  even though the failure mode exists in the broader distribution — a
  training-set-diversity gap, not a "too easy" or "already solved" ceiling.

## Fix distractor sentences colliding with the protagonist's name
**2026-07-22**

- **Fixed:** `synthetic_arithmetic`'s distractor generator could pick the
  protagonist's own name, producing self-contradictory problems (e.g. "Bob
  has 18 stickers... Bob has 1 stickers."). Found via a real training run
  (`full_claude_bigtrain.yaml`, 64 train examples): the one test failure out
  of 16 turned out to be exactly this case, not a genuine Haiku reasoning
  gap. Distractor name selection now excludes the protagonist.
- New regression test generates 200 examples with `distractor_rate: 1.0`,
  `max_distractors: 2` and asserts the protagonist is never restated.

## Add full_claude_bigtrain.yaml: does the val/test ceiling hold at scale?
**2026-07-22** · [PR #1](https://github.com/baileyrd/rusty_SkillOpt/pull/1)

- **Added:** `configs/full_claude_bigtrain.yaml` — same difficulty knobs as
  `full_claude.yaml` but `train_size` bumped from 24 to 64 (`epochs` dropped
  1 -> 1 to avoid compounding the size increase with a second epoch's calls).
- Run result: still 0/16 steps accepted, val 1.0 -> 1.0 (Haiku scored every
  single training example correctly too) — the ceiling from `full_claude.yaml`
  holds regardless of training-set size; it's the difficulty level, not an
  artifact of a small, easily-saturated set. Test score came in at 0.938
  (15/16), and the one failure turned out to be the distractor-collision bug
  fixed above, not a real generalization gap.
- First PR merged through the new PR-against-`main` workflow.

## Apply repo-config governance scaffolding; fix formatting
**2026-07-22**

- **Added:** standard governance files (SECURITY.md, CONTRIBUTING.md,
  CODE_OF_CONDUCT.md, CHANGELOG.md, RELEASE_NOTES.md, ARCHITECTURE.md,
  `docs/adr/0001-template.md`, PR/issue templates, `.github/workflows/ci-rust.yml`
  running `cargo fmt --check` / `clippy -D warnings` / `cargo test`) via the
  repo-config skill. README was left as-is (already existed).
- **Fixed:** ran `cargo fmt --all` across the workspace — it wasn't previously
  formatted to rustfmt defaults, which would have made the new CI workflow
  red on its first run.
- **Known limitation:** `ARCHITECTURE.md`'s boundary table and overview were
  hand-written for real; the ADR log is still just the seed template — no
  individual decisions have been logged yet. The CI workflow isn't wired up as
  a required branch-protection check yet (needs to happen on GitHub directly).

## Add full_claude.yaml: example.yaml-sized run against real Anthropic API
**2026-07-22** · [062e82f](https://github.com/baileyrd/rusty_SkillOpt/commit/062e82f)

- **Added:** `configs/full_claude.yaml`, mirroring `example.yaml`'s train/env
  sizing (24/8/16 examples, 2 epochs, batch_size 4, val_batch_size 8) but
  wired to the live Anthropic API instead of the mock backends.
- Run result: 0/12 steps accepted, val score 1.0 -> 1.0, test score 1.0 — the
  benchmark at this difficulty/scale was already too easy for the Haiku
  executor, so there was nothing for the gate to accept. Confirmed the full
  12-step loop executes correctly against the live API, including graceful
  recovery from one batch where the optimizer's JSON response was missing a
  required field.

## Add multi-step chains + more distractors to synthetic_arithmetic
**2026-07-22** · [974e77c](https://github.com/baileyrd/rusty_SkillOpt/commit/974e77c)

- **Added:** `multi_step_rate` (chains 2-3 sequential gain/lose/double/halve
  operations) and `max_distractors` (more than one irrelevant sentence per
  problem) on `SyntheticArithmeticParams`, plus `configs/smoke_claude_hard.yaml`
  exercising them.
- Defaults unchanged (`multi_step_rate: 0.0`, `max_distractors: 1`), so prior
  behavior is preserved unless a config opts in to the harder difficulty.
- Run result against the live API (Haiku executor/reflector, Sonnet
  optimizer): initial val score 0.75, optimizer proposed an edit telling the
  agent to filter irrelevant entities and apply multi-step operations in
  order, validation gate accepted it (val 0.75 -> 1.0), test score 1.0 —
  first real accepted-edit demonstration of the loop end to end.

## Switch reqwest to native root store; add real-Claude smoke config
**2026-07-22** · [3c8ec99](https://github.com/baileyrd/rusty_SkillOpt/commit/3c8ec99)

- **Fixed:** the default `rustls-tls` reqwest feature bundles a fixed
  webpki-roots trust store, which didn't include this environment's
  TLS-terminating egress proxy CA and made every outbound request fail with
  `UnknownIssuer`. Switched to `rustls-tls-native-roots` (reads the OS trust
  store, which already carries the proxy's CA here) — TLS verification was
  never disabled.
- **Added:** `configs/smoke_claude.yaml`, a small real-Anthropic-backed config
  used to confirm rollout/reflect/optimize calls succeed end to end against
  the live API.

## Implement rusty_skillopt: hand-rolled Rust reimplementation of SkillOpt's core loop
**2026-07-22** · [d7c078f](https://github.com/baileyrd/rusty_SkillOpt/commit/d7c078f)

- **Added:** initial Cargo workspace (`skillopt-core`, `skillopt-model`,
  `skillopt-envs`, `skillopt-cli`) implementing the rollout -> reflect ->
  aggregate/select -> optimize -> validation-gate training loop for markdown
  skill documents. Anchor-based skill-edit engine, Anthropic + OpenAI-compatible
  `ChatBackend` adapters plus a network-free Mock, a deterministic synthetic
  arithmetic `Environment`, and the `skillopt train`/`eval` CLI.
- **Known limitation, stated explicitly:** this is an independent design, not
  a line-by-line port of SkillOpt's Python source (not available to
  transcribe from). WebUI, additional providers, and the offline "Sleep"
  engine are out of scope for this pass — see README's Scope section.
- 26 tests, including an end-to-end scripted-backend test proving the loop
  accepts an edit that measurably improves validation score and rejects ones
  that don't.
