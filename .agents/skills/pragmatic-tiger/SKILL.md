---
name: pragmatic-tiger
description: >-
  Pragmatic TigerStyle-derived engineering judgment for this Rust and Bevy
  project. Use after bevy-019 and rust-modern when designing or reviewing
  architecture, workload and resource bounds, performance budgets,
  dependencies, invariants, operational behavior, documentation, and
  technical-debt decisions. Do not use it to choose Rust idioms or Bevy APIs.
---

# Pragmatic Tiger

Apply TigerStyle's useful engineering pressure without importing its Zig or
database-specific absolutes. Add project-level judgment around the existing
language and framework guidance; do not restate it.

## Precedence And Ownership

Read `bevy-019` first and `rust-modern` second.

- Let `bevy-019` own Bevy versions, features, ECS concepts, schedules, assets,
  rendering, and framework testing boundaries.
- Let `rust-modern` own Rust syntax, types, APIs, arithmetic, error handling,
  code organization, Cargo conventions, and Rust test mechanics.
- Use this skill only for cross-cutting design tradeoffs and operational
  discipline not decided by those skills.
- Follow the user's request and repository instructions before all three
  skills.
- If guidance appears to overlap or conflict, stop applying this skill at that
  boundary and follow the owning skill. Do not duplicate the rule here.

## Scale Rigor To Risk

Optimize for correctness and user data, predictable responsiveness, and
developer experience, in that order. Scale the ceremony to consequence and
reversibility:

- Keep low-risk, local, reversible work simple.
- Spend design effort before changes that affect saved data, replay behavior,
  cross-system state, public contracts, or persistent performance costs.
- Prefer the smallest design that satisfies known constraints. Do not build a
  generalized subsystem for hypothetical requirements.
- Record assumptions whose failure would force a redesign.

Reject both perfectionism and knowingly fragile delivery. Fix understood
correctness, security, data-loss, and runaway-resource hazards before building
on top of them.

## Bound Work And Growth

Identify anything that can grow with input or time: queues, retries, spawned
objects, searches, generated content, loaded assets, save data, logs, caches,
and background tasks.

- Define a domain-based limit when exhaustion or runaway work is plausible.
- Define what happens at the limit: reject, defer, shed, truncate, evict, or
  surface an error. Never let the behavior be accidental.
- Process potentially large work incrementally when doing it all at once could
  harm responsiveness.
- Give custom loops a visible termination condition or progress measure.
  Framework-owned application loops are intentionally long-lived and exempt.
- Observe or report meaningful pressure near important limits so a bound does
  not become a silent failure point.

Do not add arbitrary limits to tiny, trusted, or naturally finite work. State
why a bound is unnecessary when that conclusion is non-obvious.

## Protect Invariants At Boundaries

- Identify where state changes authority across subsystem, persistence, asset,
  user-input, network, or modding boundaries.
- Give each consequential boundary an explicit contract covering assumptions,
  ownership, relevant limits, and rejection or recovery policy.
- When corruption would be costly, verify a critical property at both creation
  and consumption through independent paths when practical.
- Review the positive space, negative space, and transition between them when
  defining acceptance criteria.

Do not impose an assertion quota or assert every argument and return value.
Leave concrete validation, failure mechanisms, and tests to `rust-modern` and
framework boundary mechanics to `bevy-019`; this skill only requires an
explicit contract and risk-proportionate independent evidence.

## Budget Performance Pragmatically

Make a rough resource sketch before implementing a subsystem with credible
frame-time, loading-time, memory, storage, network, or build-size risk.

- Name the relevant budget and expected order of magnitude.
- Seek structural wins first: less work, less data, better locality, fewer
  transitions, or amortized setup.
- Measure once executable behavior exists. Revise the sketch when evidence
  disagrees.
- Optimize hot paths and recurring spikes, not speculative micro-costs.
- Batch work only when it improves total cost without harming latency,
  correctness, or clarity.
- Reduce allocation churn in demonstrated hot paths, but allow ordinary dynamic
  allocation everywhere it is appropriate.
- Make a library default explicit when correctness or a performance budget
  depends on it. Otherwise, intentional framework defaults are acceptable.

Leave Bevy feature selection and renderer configuration to `bevy-019`.

## Treat Dependencies And Tools As Tradeoffs

Dependencies and additional tools are allowed.

- Add one when its capability outweighs security exposure, maintenance,
  compile time, binary size, platform support, and team-learning cost.
- Prefer an existing project tool for small jobs when it is adequate.
- Add a specialized tool when it clearly reduces total complexity or risk.
- Remove unused dependencies and obsolete tooling rather than carrying them by
  inertia.

Leave Cargo mechanics and Bevy feature mechanics to their owning skills.

## Preserve Local Reasoning

- Keep one authoritative representation of mutable state. Treat caches and
  views as derived state with an explicit invalidation owner.
- Calculate and validate values near their use when distance would create a
  stale-check or wrong-variable risk.
- Minimize the number of independently changing concepts a reviewer must track
  at once.
- Centralize orchestration when it makes the sequence and failure policy easier
  to inspect; keep subordinate calculations focused.

## Explain Decisions

- Choose precise domain nouns and verbs. Include units or qualifiers in a name
  when the type does not already make them unambiguous.
- Comment why a constraint, bound, ordering rule, or surprising default exists.
  Do not narrate obvious code.
- Explain the goal and method of a non-obvious test or benchmark.
- Keep important design rationale near the code, test, or configuration it
  governs so later changes encounter it.
- Use formatting and identifier casing selected by the Rust toolchain and
  `rust-modern`; do not import Zig aesthetics.

## Manage Debt Deliberately

Do not claim zero technical debt. Distinguish a safe, explicit compromise from
an unaddressed correctness hazard.

- Describe a temporary compromise, its bounded impact, and the condition that
  should trigger removal.
- Track follow-up work in the repository's normal system; do not rely on a vague
  comment alone.
- Fix hazards whose cost or blast radius compounds with further development.
- Defer speculative polish that does not improve a current constraint.

## Review Checklist

Before implementation:

1. Identify the failure modes and irreversible outcomes.
2. Identify inputs or state that can grow and decide whether they need bounds.
3. Identify authoritative state and trust boundaries.
4. Sketch resource budgets only where credible risk justifies it.

Before completion:

1. Verify behavior at important limits and trust transitions.
2. Check that overload and exhaustion behavior is intentional.
3. Measure the budgets that materially affect acceptance.
4. Record any accepted compromise and its removal trigger.
5. Confirm that no rule here overrides or duplicates `bevy-019` or
   `rust-modern`.

## Explicit Non-Goals

Do not import these TigerStyle rules into this project:

- Zero dependencies or zero new tools.
- Static startup-only allocation or a general ban on allocation.
- A ban on recursion or architecture-sized integer types.
- Assertion quotas, asserting every function value, or mandatory paired
  assertions for every property.
- Fixed function-length or line-width limits beyond tool-enforced formatting.
- Mandatory splitting of compound conditions or adding an `else` to every
  branch.
- Explicitly spelling every library default when the default is intentional.
- Zig naming, acronym capitalization, pointer initialization, formatting, or
  tooling conventions.
- Universal batching or control-plane/data-plane architecture.
- Premature optimization justified only by analogy to a storage engine.
