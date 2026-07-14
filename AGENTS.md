# Repository Instructions

## Mandatory Skills

- Invoke and read `$bevy-019`, then `$rust-modern`, then `$pragmatic-tiger` at the start of every task in this repository.
- Before every shell command, tool call, code change, or code review, confirm that all three skills are loaded for the current turn and apply their relevant guidance to that action.
- Do not skip a mandatory skill for commands that appear unrelated to Bevy or Rust. If any skill is unavailable, stop and report the problem before running commands or changing files.

## Skill Storage

- Store every custom skill created or maintained for this project under `.agents/skills/<skill-name>` inside this repository.
- Never create, copy, or move a project-specific skill into `$HOME/.codex/skills`, `$HOME/.agents/skills`, or another user-, system-, or machine-level skill directory.
- When using `$skill-creator`, pass the repository's `.agents/skills` directory as the explicit output path.
- Before creating or moving a skill, resolve its destination and verify that it remains inside this repository.
- Use a user- or system-level destination only when the user explicitly requests a skill that applies outside this project.

## Skill Precedence

- Follow the user's request and repository instructions first.
- Let `$bevy-019` exclusively own Bevy APIs, ECS, scheduling, assets, rendering, and framework-specific testing guidance.
- Let `$rust-modern` exclusively own Rust language idioms, types, APIs, arithmetic, errors, code organization, Cargo conventions, and Rust testing mechanics.
- Use `$pragmatic-tiger` only for complementary project-level decisions about risk, bounds, performance budgets, dependencies, invariants, documentation, and technical debt.
- If `$pragmatic-tiger` overlaps or conflicts with either specialized skill, ignore its guidance at that boundary and follow the specialized skill.

## Repository Scope

- Keep the project target-agnostic unless the user or repository policy explicitly selects supported targets.
- The current local development host is Windows (`x86_64-pc-windows-msvc`). Use it for fast local verification, not as a statement of product support.
- Apply the skills' independent Bevy, Rust, and project-level engineering guidance according to the ownership rules above.
- Keep shared code free of platform assumptions and isolate target-specific integration behind narrow boundaries.
- Do not add or remove target-specific features, dependencies, or configuration without an explicit target requirement.

## Validation

- Run Cargo commands from `game` unless the task requires another working directory.
- Format with `cargo fmt --all --check`.
- Validate ordinary local changes with `cargo build --workspace` on the current host.
- Use `--target` only when the task or repository policy selects a target, and do not claim compatibility for untested targets.
