# `ptime` Specification

## 1. Overview
`ptime` is a Rust-based command-line utility that recursively scans a directory tree for JPEG images and reports temporal insights derived from their EXIF metadata. Three subcommands provide targeted outputs:

- `ptime oldest [dir]` – show the earliest photo capture date.
- `ptime latest [dir]` – show the most recent photo capture date.
- `ptime hist [dir] [--width N]` – render an ASCII histogram of photo counts per year.

The tool defaults to scanning the current working directory when `[dir]` is omitted. All results are reported using paths relative to the scan root. Files without a usable EXIF timestamp are silently ignored.

## 2. Requirements

### 2.1 Functional
- Support the subcommands and arguments listed above; `hist` accepts an optional `--width N` flag before the optional directory argument.
- Recurse through all subdirectories from the scan root, considering files whose extension (case-insensitive) is `.jpg` or `.jpeg`.
- Extract capture timestamps using EXIF metadata with the following fallback order:
  1. `DateTimeOriginal`
  2. `CreateDate` (`DateTimeDigitized`)
  3. `ModifyDate`
- Ignore files that lack all three timestamps; do not mention them in output.
- For `oldest`/`latest`, choose a single photo:
  - Primary comparison on capture date (earliest or latest).
  - Tie-break by lexicographically smallest relative path.
  - Output format: `relative/path.jpg YYYY-MM-DD`.
- For `hist`:
  - Aggregate counts per year across all photos with valid timestamps.
  - Include every year between the minimum and maximum capture year, even when count is zero.
  - Render each line as `YYYY ███ 42`, using Unicode full block (`U+2588`).
  - Scale bar lengths so the longest bar fits within `width` characters.
    - Default width: `50`.
    - User-specified width: `1..=200` via `--width N`; clamp values above 200 down to 200, reject non-positive integers.
    - Empty years show no blocks before the count (e.g., `2021  0`).
- Emit nothing (no text, exit code `0`) when no photos with usable timestamps are found.

### 2.2 Non-Functional
- Emphasize performance:
  - Use Rust for implementation.
  - Avoid unnecessary allocations and metadata passes; prefer single traversal.
  - Minimize external dependencies; use efficient EXIF parsing.
- Provide clear error messages on invalid usage (e.g., bad width flag, unreadable directory).
- Exit codes:
  - `0` on success (even with empty results).
  - Non-zero on argument parsing errors, IO errors reading directories, or EXIF parsing failures that prevent processing.

## 3. Architecture & Design

### 3.1 High-Level Flow
1. Parse CLI arguments using a structured parser (e.g., `clap` or `argopt`).
2. Resolve scan root:
   - Use provided directory or `.` when omitted.
   - Canonicalize the root to enable stable relative paths.
3. Traverse directory tree depth-first or breadth-first using `walkdir` or a custom iterator; skip non-files and non-JPEG extensions.
4. For each JPEG:
   - Read EXIF data using `exif` crate (supports JPEG natively).
   - Extract timestamps in fallback order; convert to `chrono::NaiveDate`.
   - Collect per-command data:
     - For `oldest`/`latest`: update running extrema.
     - For `hist`: increment year counts.

### 3.2 Module Structure
- `main.rs` – CLI entrypoint; dispatches subcommands.
- `cli.rs` – argument parsing and validation (ensure `--width` bounds, order handling).
- `scanner.rs` – directory traversal yielding `(PathBuf, NaiveDate)` using streaming iterator.
- `analysis.rs` – functions to compute oldest/latest image and histogram dataset.
- `render.rs` – histogram ASCII rendering respecting configurable width.
- `error.rs` – unified error types (using `thiserror` or custom enum) mapped to meaningful exit codes.

### 3.3 Data Structures
- `PhotoMeta { rel_path: PathBuf, date: NaiveDate }` for traversal output.
- Histogram data: `BTreeMap<i32, usize>` to maintain sorted years.
- Stats accumulator struct for oldest/latest with fields `(Option<NaiveDate>, Option<PathBuf>)` or custom struct implementing comparison logic.

### 3.4 External Crates
- `clap` (v4) for CLI parsing with subcommands and flags.
- `walkdir` for recursive directory traversal with symlink handling.
- `exif` for reading EXIF metadata.
- `chrono` for date parsing/formatting.
- (Optional) `anyhow`/`thiserror` for ergonomic error handling.

### 3.5 Performance Considerations
- Single-pass traversal feeding command-specific accumulators; avoid storing entire photo list when unnecessary.
- Lazy EXIF parsing: read enough tags to obtain timestamps; handle files lacking EXIF gracefully.
- Use relative paths computed once from canonical root via `pathdiff` or manual stripping.

## 4. Data Handling Details
- Only open files identified as JPEG via extension check; optionally confirm MIME signature for robustness.
- EXIF timestamp parsing:
  - EXIF values come as `YYYY:MM:DD HH:MM:SS`; parse and convert to `NaiveDate` by truncating time.
  - Reject dates that fail parsing; treat file as lacking usable timestamp.
- Relative paths:
  - After canonicalizing root, derive relative paths using `path.strip_prefix(root)`; preserve directory separators.
- Histogram scaling:
  - Determine `max_count` across years.
  - Compute `scale = width / max_count` as `f64`; for each year, `blocks = max( (count * scale).round() as usize, 0)`; ensure non-zero counts render at least one block.
  - Empty years (count `0`) should render spaces only before the count.

## 5. Error Handling Strategy
- CLI validation errors (unknown command, invalid width, non-existent directory) → descriptive message to stderr, exit code `2`.
- IO errors reading directories or files → bubble up with context (`Failed to read /path/to/file: <reason>`), exit code `3`.
- EXIF parsing failures -> log at debug verbosity only if logging enabled; treat as missing timestamp.
- Unexpected internal errors → fallback to generic message with exit code `1`.
- No output for valid runs that simply find no dated photos.

## 6. Testing Plan

### 6.1 Unit Tests
- `scanner` module:
  - Handles JPEG extension filtering (case-insensitive).
  - Properly strips root prefix to produce relative paths.
- `analysis` module:
  - Oldest/latest selection with tie-breaking on paths.
  - Histogram aggregation covers all years between min/max.
- `render` module:
  - Scaling logic caps at requested width and clamps >200.
  - Years with zero counts render empty bars.

### 6.2 Integration Tests
- Use temp directories with fixture images (embed minimal JPEGs with crafted EXIF data using in-test generation or fixtures).
- Scenarios:
  - Mixed valid/invalid timestamps (invalid ones skipped).
  - Oldest/latest with same date different names.
  - Histogram default width vs custom width.
  - Directory argument override and relative path correctness.
- Validate CLI outputs match expected strings.

### 6.3 Performance Checks
- Benchmark traversal on synthetic dataset (e.g., 10k JPEGs) ensuring no excessive memory usage.

### 6.4 Manual QA Checklist
- Run commands on a directory without JPEGs (expect no output, exit 0).
- Run `ptime hist --width 300` (expect clamp to 200).
- Verify behavior when directory is unreadable (proper error to stderr).

## 7. Delivery Notes
- Target Rust edition `2021`.
- Provide a `Cargo.toml` with relevant dependencies.
- Document usage in `README.md` after implementation (future work).

