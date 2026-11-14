# AGENTS.md

Working directory: `D:\rust\webspice`. Project name: `webspice` (exact spelling).
Use paths relative to the working directory, never absolute paths.

Rust library (LTspice `.asc`/`.asy` parser, WASM export) plus a Stencil/TypeScript
web renderer in `web/`. This file defines how the agent works. It contains no project content.
All project knowledge (purpose, formats, AST, rendering rules, structure, test cases)
lives in `docs/documentation.md`. That file is the single source of truth.

## Documentation protocol (highest priority)
1. At the start of a session, read `docs/documentation.md` fully, once. Do not re-read it
   later in the same session; before changing a section, re-read only that section.
2. For every new requirement, changed behavior, new flag, or new constraint:
   update `docs/documentation.md` in the same task as the code change.
3. Write the docs change first, then implement against it.
4. If a request contradicts the docs, stop and ask one specific question. Do not silently
   override the docs and do not silently ignore the request.
5. Docs describe the current state only. Record changes in the "Changelog" section
   (one entry per change, concise, newest first), not as history inside other sections.
6. Put project facts only in the docs. Put agent rules only in this file. Never duplicate
   content between the two; link by section name instead.
7. A task is not done until the docs match the code. Compare the sections you touched
   with the code before finishing.
8. Do not edit `README.md` unless asked. If a change makes it outdated, say so in your summary.

## Commands
Rust (from the repo root):
- Check + lint (includes type checking, use this instead of `cargo check`):
  `cargo clippy --all-targets --message-format=short -q -- -D warnings`
- Build: `cargo build -q`
- Test: `cargo test -q`
- Single test: `cargo test -q <name> -- --nocapture`
- Format: `cargo fmt --all`

Web (from `web/`; scripts: docs section "Build / Run (npm, aus `web/`)"):
- Unit tests: `npm run test`
- Full build: `npm run build`
- After changing Rust code used by the WASM export: `npm run build-wasm` first.

Output rules:
- Always use the `-q` and `--message-format=short` flags shown above.
- Fix the first reported error only, then run the command again.
- Never run the same failing command again without changing something first.

## Tasks
- A sequential task queue lives in `tasks/`: one file per pending task
  (`NN-<name>.md`, ordered by the number prefix) plus `tasks/tasklog.md`.
- `tasks/tasklog.md` is the source of truth for order and status; the task files
  hold the full spec for each work item.
- When asked to implement work, work tasks strictly one at a time in order.
  Implement only the current task; do not start the next one in the same session.
- When a task is finished (code done, docs in sync, clippy/tests/fmt clean):
  record the date and time under the task's headline in `tasks/tasklog.md`
  (moved to "Completed") and delete the task file. Only then may the next task start.

## Workflow
1. Read docs (once), then the relevant code. Do not guess APIs.
   Locate with search first, then read only the needed range. Do not re-read unchanged files.
2. Update docs (see protocol), then make the smallest code change that satisfies them.
3. After every change: run the clippy command (Rust) or the unit tests (web), and only
   the tests related to the change.
4. Before finishing: full `cargo test -q`, `cargo fmt --all`, clippy clean,
   `npm run test` in `web/` if web code or the WASM export changed, docs in sync.
5. If a command fails, fix the cause. Never weaken tests or lints.
6. If file tools disagree about what exists (read vs. glob vs. shell), verify with the
   shell (`Get-ChildItem`) before concluding a file is missing.

## Editing
- Change files with targeted edits (replace only the changed lines).
- Never rewrite a whole file to change a few lines. This applies to code and to the docs.

## Code rules
Rust:
- No `unsafe`. No `unwrap()` / `expect()` outside tests. Use `?` with the library's `Error` type.
- Validate input before indexing. Return a clear error on bad files; never panic.
- `#[allow(...)]` requires a comment with the reason.
- Public items get `///` doc comments.

TypeScript (`web/`):
- `web/src/render/render.ts` stays pure (AST → SVG DOM, no browser globals beyond the
  passed host). Every variable render value (fonts, colors, stroke, toggles) comes from
  `render-config.json`; see docs section "Render-Konfiguration".
- No `any`; keep `web/src/types/ast.ts` in sync with the JSON contract
  (docs section "WASM / Web-Export").

## Testing
- Rust unit tests next to code in `#[cfg(test)] mod tests`; integration tests use
  `tests/fixtures/` and `tests/asy-fixtures/`.
- Web unit tests live in `web/src/render/render.spec.ts` (jsdom).
- Use small synthetic data generated in the test.
- Cover every case listed in the docs section "Required test cases" when touching related logic.
- New behavior and bug fixes require a test. For bugs: write the failing test first.

## GUI verification
- A GUI task (any change to layout, theming, rendering, or widget state) is not finished
  until a screenshot of the result exists in `web/screenshots/`, is listed in the docs
  section "Evidence (GUI-Abnahme)", and has been compared against the docs rendering rules
  (and against the LTspice reference screenshot when the task is about fidelity).
- Screenshots are evidence: do not claim a GUI change looks right without one.
  If you cannot create a screenshot, say so explicitly.

## Dependencies
- No new dependencies (cargo or npm) without asking.

## Git and files
- Never commit, push, or switch branches unless asked.
- Never edit `Cargo.lock` or `package-lock.json` by hand.
- Do not touch: `target/`, `.git/`, `web/node_modules/`.
- Generated, never edit by hand: `web/www/` and `web/wasm/` (rebuild via the build
  scripts; docs section "Build / Run (npm, aus `web/`)"). Committing `web/www/` is
  allowed only as part of the deployment tasks.
- `sym/` (LTspice symbol corpus) is read-only. `web/webspice-symbols/` may be changed
  only as part of a task and must be reflected in the docs.

## Boundaries
- Ask before: adding dependencies, changing the public API or the JSON/WASM contract,
  changing documented semantics.
- No unrelated refactoring.

## Communication
- Concise: what changed, why, which doc sections were updated.
- If ambiguous, ask one specific question instead of guessing.
- If you could not run or verify something, say so explicitly.
