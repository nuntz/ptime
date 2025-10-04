# ptime Development Blueprint

## Blueprint Overview
- Establish a Rust 2021 binary crate with a library module layout that keeps CLI orchestration thin and moves logic into testable modules.
- Provide a clap-driven CLI surface that validates subcommands and arguments (including `--width`) and reports helpful usage errors.
- Build a single-pass filesystem traversal that canonicalizes the scan root, filters JPEGs case-insensitively, and produces relative paths that stay stable across runs.
- Extract EXIF timestamps with graceful fallbacks, returning `PhotoMeta { rel_path, date }` entries while skipping files that lack usable dates.
- Implement analysis functions for oldest/latest selection and histogram aggregation, alongside an ASCII renderer that respects configurable width limits.
- Integrate the pipeline in `main`, map errors to exit codes, and ship with comprehensive unit/integration tests plus developer-ergonomics commands (`cargo fmt`, `cargo clippy`).

## Iterative Chunk Breakdown
### Chunk 1 – Foundational Scaffolding
- Initialize the Cargo binary crate, configure edition 2021, and declare shared library modules (`cli`, `scanner`, `metadata`, `analysis`, `render`, `error`).
- Add core dependencies (`clap` with derive, `walkdir`, `exif`, `chrono`, `thiserror`, test helpers) and baseline CI-friendly scripts (`cargo fmt`, `cargo clippy`, `cargo test`).
- Provide placeholder structs/functions and smoke tests so the crate builds and tests run green from the start.

### Chunk 2 – CLI Surface & Validation
- Define the `Cli` parser with subcommands (`Oldest`, `Latest`, `Hist`) and optional directory argument defaulting to `.`.
- Implement `--width` parsing for `hist`, enforcing numeric bounds (1..=200, clamping above 200) with descriptive error messages.
- Expose a `CliCommand` enum and conversion helpers, with unit tests covering happy paths and failure cases.

### Chunk 3 – Filesystem Scanning Backbone
- Canonicalize the scan root, compute relative paths, and recursively walk directory trees via `walkdir`.
- Filter files by `.jpg` / `.jpeg` extension (case-insensitive) and ignore non-regular files or unreadable entries gracefully.
- Return lightweight `FoundFile` records (absolute + relative path) and cover behaviors with temp-directory tests.

### Chunk 4 – Metadata Extraction & Photo Records
- Introduce a `metadata` module that reads EXIF fields in the required fallback order, parsing to `NaiveDate`.
- Combine scanning + metadata into a `scan_photos` routine that yields `PhotoMeta` records and silently skips files without timestamps.
- Add fixtures with crafted EXIF data to prove fallback order, bad timestamps, and missing tags.

### Chunk 5 – Analysis & Rendering
- Implement `analysis` helpers for oldest/latest selection (with path tie-breaks) and histogram aggregation that fills missing years.
- Create the histogram renderer that scales block widths (default 50, clamp to provided width, zero-year formatting) using Unicode full blocks.
- Unit-test edge cases: identical timestamps, empty collections, width extremes, zero-count years.

### Chunk 6 – Integration, Error Mapping, & Hardening
- Wire the CLI command handlers to scanner/metadata/analysis/render modules and print results in the required formats.
- Map domain errors to exit codes (`2` usage, `3` IO, `1` fallback), ensuring commands with no valid photos exit 0 with no output.
- Add integration tests using fixtures (via `assert_cmd`/`tempfile`) plus final clippy/fmt gating, README usage notes, and TODOs for future perf work.

## Fine-Grained Steps & Safety Check
1. Bootstrap the crate (`cargo init --bin ptime`), set edition 2021, add dependency stubs, scaffold module files, and ensure `cargo fmt`, `cargo clippy -- -D warnings`, and `cargo test` succeed with placeholder code.
2. Implement the `cli` module with clap derive structs/enums, default directory resolution, width validation, and unit tests covering happy path, out-of-range width, and invalid command scenarios.
3. Build the `scanner` module to canonicalize roots, walk directories using `walkdir`, filter JPEG extensions case-insensitively, compute relative paths, and unit-test using temp directories with mixed files and symlinks.
4. Add a `metadata` module that opens JPEGs, reads EXIF via the fallback order (`DateTimeOriginal`, `CreateDate`, `ModifyDate`), parses to `NaiveDate`, exposes `PhotoMeta`, integrates with `scanner::collect`, and unit-tests using fixture files for valid, fallback, and missing metadata.
5. Implement `analysis` helpers (`find_oldest`, `find_latest`, `build_histogram`) honoring tie-breakers and year padding, with exhaustive unit tests on small in-memory `PhotoMeta` sets.
6. Implement the histogram renderer to scale bars to width, clamp width >200, guarantee at least one block for non-zero counts, and unit-test zero-count, sparse, and max-width edge cases.
7. Integrate everything in `main`: parse CLI, dispatch commands, call scanner/metadata/analysis/render, print required output formats, handle empty results (emit nothing), and add integration tests via `assert_cmd` + fixtures verifying each subcommand and error handling.
8. Final hardening: introduce structured error types (`thiserror`), map to exit codes, document usage in README, double-check performance assumptions, and run `cargo fmt`, `cargo clippy`, `cargo test` as release gates.

Step size review: Each step delivers a functional increment guarded by targeted tests, while remaining small enough to isolate failures and refactor safely.

## Prompt Series
### Prompt 1 – Project Skeleton & Scaffolding
```text
You are implementing Step 1 of the ptime plan (see SPEC.md for requirements). Goals:
1. Initialize a Rust 2021 binary crate named `ptime` if it does not exist; if it does, ensure the structure matches the plan.
2. Configure `Cargo.toml` with placeholder dependencies: `clap` (with `derive`), `walkdir`, `exif`, `chrono`, `thiserror`, `anyhow` (or equivalent error helper), plus dev-dependencies `assert_cmd`, `predicates`, `tempfile`.
3. Create `src/main.rs` delegating to `ptime::run() -> anyhow::Result<()>` in `src/lib.rs`.
4. Add module files: `src/cli.rs`, `src/scanner.rs`, `src/metadata.rs`, `src/analysis.rs`, `src/render.rs`, `src/error.rs`, each with stub structs/functions that compile.
5. Add a smoke unit test (e.g., `lib.rs`) ensuring `run()` returns `Ok(())` while everything is still a stub.

Development approach:
- Write the stubs and tests first, run `cargo fmt`, `cargo clippy -- -D warnings`, then `cargo test` to confirm the baseline passes.
- Keep stubs minimal (e.g., `todo!()` is fine if covered by tests that expect `Ok(())`), but ensure `cargo test` remains green.

Deliverable: compiling crate with module skeleton and passing basic tests.
```

### Prompt 2 – CLI Parsing & Validation
```text
You are implementing Step 2 of the plan. With the scaffolding in place, add real CLI parsing logic.

Goals:
1. In `src/cli.rs`, define clap-derived structs/enums covering:
   - `Cli` root parser with subcommand requirement.
   - `Command` enum with variants `Oldest`, `Latest`, `Hist`.
   - Shared optional `PathBuf` directory argument defaulting to `.` when omitted.
   - `Hist` variant that accepts `--width <N>` before the optional directory.
2. Enforce width handling: accept integers 1..=200, clamp values >200 to 200, and emit a clear error for non-positive integers.
3. Expose a `parse()` helper returning a strongly-typed `CliCommand` struct (your design) for downstream modules.
4. Update `src/lib.rs` (or a new `run_cli()` helper) to use the parser while still returning `Ok(())`.
5. Add unit tests in `src/cli.rs` validating:
   - Happy paths for each subcommand (with/without dir, width default vs custom vs >200).
   - Error on zero/negative width.
   - `--help` still works (use clap's testing helper).

Testing workflow: add failing tests first, implement parsing/validation to pass them, then run `cargo fmt`, `cargo clippy -- -D warnings`, and `cargo test`.

Ensure no other modules gain unfinished code at this stage.
```

### Prompt 3 – Filesystem Scanner
```text
Implement Step 3: the filesystem scanning backbone, independent of EXIF parsing.

Goals:
1. In `src/scanner.rs`, add:
   - `FoundFile { rel_path: PathBuf, abs_path: PathBuf }`.
   - `pub fn scan_candidates(root: &Path) -> Result<Vec<FoundFile>, PtimeError>` (or iterator-based if you prefer), which:
     * Canonicalizes the root (preserving the original argument for relative-path computation).
     * Recursively walks directories with `walkdir`, skipping symlink loops and unreadable entries by returning an error with context.
     * Filters files with extensions `.jpg`/`.jpeg` case-insensitively.
     * Computes stable relative paths against the canonical root.
2. Add helper functions for extension filtering and relative path derivation to keep the code testable.
3. Update `src/error.rs` with preliminary error variants used here.
4. Unit tests:
   - Use `tempfile::tempdir()` to create nested directories containing JPEG/non-JPEG files, uppercase extension, and ensure only JPEGs are returned.
   - Ensure root canonicalization works (e.g., pass `.` vs absolute path).
   - Ensure unreadable directory yields an error variant.

Follow TDD: author tests first, implement logic, then run `cargo fmt`, `cargo clippy -- -D warnings`, `cargo test`.
```

### Prompt 4 – Metadata Extraction & Photo Records
```text
Step 4 integrates EXIF parsing with the scanner.

Goals:
1. Create `src/metadata.rs` helpers:
   - `pub struct PhotoMeta { pub rel_path: PathBuf, pub date: NaiveDate }`.
   - `pub trait ExifReader` (optional) or direct functions that read EXIF from a `&Path`.
   - Implement `read_capture_date(path: &Path) -> Result<Option<NaiveDate>, PtimeError>` parsing timestamps in the fallback order (`DateTimeOriginal`, `CreateDate`, `ModifyDate`) and converting to `NaiveDate`. Skip files with missing or unparsable timestamps by returning `Ok(None)`.
2. Extend `scanner` to expose `pub fn collect_photos(root: &Path) -> Result<Vec<PhotoMeta>, PtimeError>` that composes `scan_candidates` and `read_capture_date`, filtering out `None`.
3. Ensure EXIF parsing reads only once per file and handles IO/EXIF errors gracefully (treat unreadable EXIF as a recoverable skip unless the file can't be opened).
4. Tests:
   - Add JPEG fixtures under `tests/fixtures/` with specific EXIF tags (consider embedding static byte arrays via `include_bytes!`).
   - Cover success with each fallback field, a file lacking timestamps (skipped), and a malformed EXIF date (skipped).
   - For performance, ensure we're not loading entire files unnecessarily (can assert we only open once by design review).

Run `cargo fmt`, `cargo clippy -- -D warnings`, `cargo test` (unit tests + any fixture-based tests). Keep scanner tests green.
```

### Prompt 5 – Analysis Helpers
```text
Implement Step 5 focusing on pure analysis logic.

Goals:
1. In `src/analysis.rs`, add functions:
   - `pub fn find_oldest<'a>(photos: &'a [PhotoMeta]) -> Option<&'a PhotoMeta>`.
   - `pub fn find_latest<'a>(photos: &'a [PhotoMeta]) -> Option<&'a PhotoMeta>`.
   - `pub fn build_histogram(photos: &[PhotoMeta]) -> BTreeMap<i32, usize>`, filling every year between min/max inclusive.
2. Enforce tie-breaking: when dates match, return the photo with lexicographically smallest `rel_path`.
3. Ensure histogram includes zero-count years and ignores empty input (return empty map).
4. Unit tests covering:
   - Oldest/latest with tie-breaking and empty input.
   - Histogram gaps, zero counts, and single-year scenarios.

Maintain TDD: add tests first, implement logic, run `cargo fmt`, `cargo clippy -- -D warnings`, `cargo test`.
```

### Prompt 6 – Histogram Rendering
```text
Implement Step 6: rendering histogram output.

Goals:
1. In `src/render.rs`, add `pub fn render_histogram(year_counts: &BTreeMap<i32, usize>, width: usize) -> Vec<String>` that:
   - Uses width default 50 upstream, but this function should accept the resolved width (already clamped).
   - For non-zero counts, scales bar lengths so the maximum count uses the full width, rounding to nearest integer but guaranteeing at least one block.
   - For zero counts, emit no blocks (just spaces between year and count).
   - Uses Unicode full block (`\u{2588}`) characters.
2. Provide helper `fn scale_counts(...)` if helpful.
3. Unit tests covering:
   - Empty histogram (returns empty vec).
   - Mixed counts with scaling and rounding edge cases.
   - Zero-count year formatting (e.g., `2021  0`).
   - Width = 1 and width large (e.g., 200) to ensure clamps honored upstream.

Continue TDD and run `cargo fmt`, `cargo clippy -- -D warnings`, `cargo test`.
```

### Prompt 7 – CLI Integration & End-to-End Tests
```text
Final step (Step 7 + Step 8 hardening): integrate modules and ensure everything works under the CLI.

Goals:
1. Update `src/lib.rs` / `run()` to:
   - Parse CLI input.
   - Resolve scan root (default `.`).
   - For `oldest`/`latest`, call `collect_photos`, choose the record, print `relative/path.jpg YYYY-MM-DD`, or exit 0 with no output if no photos.
   - For `hist`, resolve width (default 50, clamp >200 already handled), build histogram, render lines, print joined with `\n`, or emit nothing if no photos.
2. Enforce exit codes: usage errors (invalid width etc.) should surface as clap errors (exit 2), IO errors (e.g., unreadable directory) map to exit 3, unexpected errors to exit 1.
3. Update `src/error.rs` to map to exit codes and messages; ensure errors print to stderr without stack traces.
4. Add integration tests in `tests/cli.rs` using `assert_cmd` + fixtures to cover:
   - `ptime oldest <fixture_dir>` outputs expected path/date.
   - `ptime hist --width 5 <fixture_dir>` renders scaled histogram with zero-count year.
   - Directory with no valid photos yields empty stdout and exit 0.
   - Invalid width (0) fails with message and non-zero exit.
5. Document the testing commands (README or comments) and ensure `cargo fmt`, `cargo clippy -- -D warnings`, `cargo test` remain green.

After this prompt, the project should meet SPEC.md requirements with no orphaned code.
```
