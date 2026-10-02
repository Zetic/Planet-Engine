# Planet Engine Development Rules

These rules apply to automated and assisted development work in this repository. They are intended to keep Planet Engine changes causal, reproducible, reviewable, and CI-ready before a pull request is treated as complete.

## 1. Repository state is authoritative

- Start every development task by checking the current `main` head and open pull requests.
- Do not assume a prior branch, generated artifact, workflow state, or merged PR is still current.
- Inspect the actual source and relevant tests before proposing or implementing a fix.
- When a task depends on an earlier PR, confirm that PR's merge state and current `main` ancestry first.

## 2. Preserve the causal world-generation model

Planet Engine should explain generated outcomes through upstream physical state rather than post-hoc visual targets.

- Fix problems at the earliest causal layer that actually creates them.
- Do not force land percentage, elevation percentage, continent count, shoreline shape, or other visual targets directly unless that quantity is itself an explicit physical parameter.
- Do not use seed-specific hacks.
- Do not use plate IDs, provenance labels, or debug identities as physical authority. They may be used for diagnostics and lineage observation only.
- Prefer persistent physical state such as crust type, thickness, density, strain, age, structural history, subsidence, basin potential, thermal state, or boundary kinematics when changing generation behavior.
- Preserve established stage ownership. Do not compensate for an upstream defect by retuning an unrelated downstream stage without evidence.

## 3. Diagnose before tuning

For calibration defects:

1. Reproduce the issue with the reported seed and resolution.
2. Add temporary instrumentation when existing observability cannot identify the responsible causal field.
3. Compare the problem seed against established fixed seeds and at least one holdout.
4. Identify the causal variable or transformation responsible before changing calibration constants.
5. Remove temporary instrumentation before finalizing the PR unless it provides durable value as a permanent diagnostic.

A visual symptom alone is not sufficient justification for a physics change.

## 4. Production-resolution validation matters

- Fast/coarse tests are smoke tests, not substitutes for production behavior.
- When a bug is visible only at L6→L8 or another production path, add at least one production-resolution acceptance or holdout that can reproduce the failure.
- A seed that exposes a real uncaught regression should normally become a permanent holdout or acceptance case unless there is a strong reason not to.
- Acceptance thresholds must represent physical intent across a seed cohort, not merely make one seed pass.

## 5. Generated artifacts are part of the implementation

This repository commits generated outputs. A source change is not implementation-complete until all affected generated artifacts are refreshed and committed.

### Browser TypeScript

Changes that affect files compiled from `src/` require regeneration of committed browser output:

```bash
npm run build
```

Before final CI, verify that `dist/` matches the current source and commit the resulting changes.

### Rust/WASM browser bridge

Changes to Rust exposed through the WASM bridge, wasm-bindgen interfaces, browser protocol bindings, or any Rust code that changes the packaged browser module require regeneration of the committed WASM package:

```bash
npm run build:worldgen-wasm
```

Commit the resulting `src/wasm-worldgen/` changes.

### Required rule

**Do not wait for CI parity jobs to reveal a known stale generated artifact.**

Before the first final CI run:

1. Inspect the source diff.
2. Determine every committed generated artifact affected by that diff.
3. Regenerate those artifacts.
4. Commit them.
5. Then run or rely on the final CI gates.

CI should discover genuine regressions, not predictable source/artifact mismatch.

## 6. Pre-CI checklist

Before treating a branch as CI-ready:

- run `npm run typecheck` when TypeScript changes;
- run `npm run build` when browser source changes;
- run relevant TypeScript tests when browser behavior changes;
- run relevant Rust tests/checks when Rust changes;
- run `npm run check:worldgen-wasm` when the WASM bridge changes;
- run `npm run build:worldgen-wasm` when the committed browser WASM package can change;
- inspect `git diff` after every build step;
- ensure generated outputs are committed;
- ensure temporary diagnostic workflows/files are removed or intentionally retained;
- ensure documentation reflects any changed public diagnostic or calibration contract.

If the environment cannot execute one of these steps locally, determine the expected generated dependency before CI and use the repository's supported workflow/artifact path to refresh it. Do not classify a predictable parity failure as a product regression.

## 7. Pull-request discipline

- Keep each PR focused on one causal problem or one infrastructure/observability problem.
- Do not mix unrelated cleanup into a calibration PR unless required for correctness.
- Open calibration/physics PRs as draft while diagnostics or acceptance criteria are still changing.
- Mark a PR ready only when the exact head has the required green gates and no unresolved review threads.
- Do not merge without explicit user direction.
- Record the accepted seed cohort, production-resolution measurements, and meaningful before/after behavior in the PR description.

## 8. Visual-review workflow

Visual review of the deployed/generated browser result occurs **after merge** unless the repository gains a reliable branch-preview mechanism.

Pre-merge validation should therefore use:

- deterministic fixed-seed metrics;
- causal diagnostics;
- acceptance programs;
- browser protocol/diagnostic tests;
- full Rust causal-engine tests;
- packaged WASM parity.

Do not request screenshots as a prerequisite for merging a PR that cannot yet be exercised through the deployed Pages build.

## 9. Calibration observability

The Calibration JSON is the canonical compact browser diagnostic artifact. The LLM Summary is a readable subset/rendering of the same packet.

When adding new calibration diagnostics:

- add durable causal measurements to the canonical calibration packet first;
- keep raw L8 arrays out of the exported packet unless specifically justified;
- prefer compact aggregates that distinguish competing causal explanations;
- preserve enough information to compare emergent vs submerged, continental vs transitional vs oceanic, or other physically meaningful classes;
- include fidelity notes when browser-side aggregation is approximate;
- render only the highest-value subset in the LLM Summary.

Crash/debug reports remain separate from successful-world calibration diagnostics.

## 10. Physics changes require regression protection

A physics/calibration PR should protect both the newly fixed failure and prior accepted behavior.

Typical checks include, where relevant:

- plate count and resolvable-plate constraints;
- boundary-type balance and geometry;
- crust-class fractions;
- continental assembly morphology;
- transitional-margin width/coverage;
- continental freeboard and submerged shelf depth;
- mean land elevation and highland fractions;
- ocean depth and water-volume closure;
- orogenic localization;
- hydrology/erosion conservation;
- protocol and WASM parity.

Do not preserve an exact hash if the intended physics legitimately changes. Preserve invariants and accepted behavior instead.

## 11. Temporary diagnostics and workflows

Temporary diagnostics are acceptable during development when they answer a specific unresolved question.

Before finalization:

- remove temporary CI workflows, debug binaries, and one-off instrumentation unless they have lasting value;
- convert valuable diagnostics into permanent bounded observability or acceptance tests;
- restore standard repository workflows;
- rerun final validation on the exact cleanup head.

## 12. Final status reporting

After tool or repository work, always end the development pass with a user-facing status message.

The status must state:

- what changed;
- current branch/PR and head when relevant;
- which gates are green, red, pending, or intentionally not run;
- whether the PR is draft, ready, or blocked;
- the next concrete action.

Do not remain in silent CI polling loops. If a long-running job is still active, report the current state and stop the turn; continue polling only in a later turn if requested.

## 13. Completion definition

A Planet Engine PR is not complete merely because source code compiles.

A PR is implementation-complete only when:

1. the causal/source change is finished;
2. relevant regression coverage exists;
3. required committed generated artifacts are synchronized;
4. temporary development scaffolding is cleaned up;
5. documentation is current;
6. the exact PR head passes the required CI gates;
7. review threads are resolved;
8. final status has been reported clearly.

When any of these are missing, describe the branch as still in development rather than ready to merge.
