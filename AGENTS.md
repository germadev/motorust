# Instructions for AI Agents (AGENTS.md)

This document establishes the mandatory operational rules, documentation protocols, and coding conventions for all AI agents working on the **Motorust** repository.

---

## 1. Language Policy

- **Documentation & Agent Guidelines:** Both `AGENTS.md` and all documentation files (`README.md`, `docs/`, `RECORD/`) must be written in **English**.
- **Source Code Comments:** All code comments, docstrings (`//!` and `///`), function documentation, and inline explanations across all languages (`Rust`, `JavaScript`, `YAML`, etc.) must be written in **English**.
- **User Interactions:** If the user prompts in Spanish (or another language), communicate respectfully with the user in their preferred language, but ensure that all code comments, commits, and `RECORD/` entries remain strictly in English.

---

## 2. Mandatory Activity Recording (`RECORD/`)

Every non-trivial modification, feature development, bug fix, refactor, architectural change, or task execution **MUST** be recorded in the `RECORD/` directory.

### 2.1 Append-Only vs. Read-Only Discipline
- **`.WIP.md` files are append-only:** While work on a topic is in progress, new entries must be appended to the active `.WIP.md` file. Never delete, truncate, or rewrite historical entries.
- **`.completed.md` files are strictly read-only:** Once a milestone or task has concluded, passed tests, and been renamed to `.completed.md`, it becomes an immutable, read-only historical record. Never edit, modify, or append to a `.completed.md` file. Any subsequent work, iterations, or new tasks must be tracked in a new `.WIP.md` file.

### 2.2 File Naming Convention & Status Suffixes
- Files in `RECORD/` MUST follow the format:
  ```
  RECORD/YYYY-MM-DD.<topic-slug>.<status>.md
  ```
- Every file in `RECORD/` **MUST** end with either `.WIP.md` or `.completed.md`:
  - **`.WIP.md` (Work In Progress):** Append-only log used while work on that topic or milestone is currently active and ongoing.
  - **`.completed.md`:** Applied once the task, feature, or milestone has been concluded, verified (`cargo test`), and finalized. When finishing an active task, rename the corresponding `.WIP.md` file to `.completed.md`. Once finalized, `.completed.md` files are strictly **read-only**.
- Examples:
  - `RECORD/2026-10-09.foundation.completed.md`
  - `RECORD/2026-10-11.split-screen-and-web.completed.md`
  - `RECORD/2026-10-12.terrain-textures-trees-and-grass-fx.completed.md`

### 2.3 Record Entry Format
Each entry/block recorded must contain:
1. **Header & Context:** Date, block/task title, and associated commit or PR hash if known.
2. **User Request:** Direct summary of what the user asked for.
3. **Implementation Details:** What was done, including files modified, algorithms used, and design rationale.
4. **Verification & Testing:** Test results (`cargo test`), format checks (`cargo fmt`), clippy status, and visual/performance notes.

---

## 3. Technical Standards & Codebase Architecture

1. **Rust & Bevy Architecture:**
   - The engine is Bevy (ECS).
   - Systems must be split cleanly across plugins (`BikePlugin`, `TrackPlugin`, `WorldPlugin`, etc.).
   - Physics and simulation run in a fixed time step (`FixedUpdate` at 120 Hz) with interpolated rendering (`RunFixedMainLoopSystems::AfterFixedMainLoop`) to ensure consistent behavior across all monitor refresh rates and hardware.

2. **Performance & GPU Batching:**
   - High frame rate and 60+ FPS performance are top priorities.
   - Reuse shared mesh handles and materials wherever possible (e.g., batched tree trunks, tree crowns, grass particles) to maximize GPU instancing.
   - Avoid allocating or spawning unbatched heavy entities in per-frame update loops.

3. **Testing & Code Cleanliness:**
   - All pure logic (physics steps, steering, terrain shaping, audio synthesis, camera math) must have unit tests.
   - Run and verify `cargo test` before concluding tasks.
   - Keep code cleanly formatted with `cargo fmt --check`.
   - Ensure `cargo clippy` passes without warnings.

---

## 4. Tools (`tools/`)

- Scripts that are useful beyond the task at hand, above all for working on the circuits (drawing a layout and its height profile, checking it against the limits the tests set), and also importers and packaging helpers, live in `tools/`: not in a scratch directory, and not inside `src/`.
- Each tool opens with a header saying what it does, how to run it and what it needs. Where it mirrors the game's own code (as `tools/circuit_layout.py` mirrors the centre line and the profile of `src/track.rs`), keep the two in step.
- List every tool in `docs/architecture.md`. One-off scratch scripts and temporary harnesses (e.g. for capturing screenshots) are not committed.
