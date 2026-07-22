# Release Notes

Tracks notable changes to this repo, reverse chronological. This repo develops
on a feature branch pushed directly to `origin` rather than through merged
PRs, so entries are keyed by commit rather than PR number.

---

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
