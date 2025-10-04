# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

`ptime` is a Rust CLI tool that analyzes JPEG photos by EXIF timestamps. It has three subcommands:
- `oldest` - find earliest photo
- `latest` - find most recent photo
- `hist` - ASCII histogram of photos by year

The codebase emphasizes single-pass efficiency, graceful error handling, and comprehensive testing.

## Development Commands

### Build and Run
```bash
# Development build
cargo build

# Release build (optimized)
cargo build --release

# Run with args
cargo run -- oldest /path/to/photos
cargo run -- hist --width 100 .
```

### Testing
```bash
# Run all tests (unit + integration)
cargo test

# Run specific test
cargo test test_find_oldest_with_tie_breaking

# Run tests in specific module
cargo test analysis::

# Run integration tests only
cargo test --test cli

# Quiet output
cargo test --quiet
```

### Code Quality
```bash
# Format code (required before commits)
cargo fmt

# Lint with clippy (must pass with -D warnings)
cargo clippy -- -D warnings

# Check without building
cargo check
```

## Architecture

### Module Structure and Responsibilities

**Data Flow Pipeline:**
```
CLI args → Scanner (find JPEGs) → Metadata (extract EXIF) → Analysis (compute results) → Render (format output)
```

**Key Modules:**

1. **`cli.rs`** - Argument parsing with clap
   - Defines `Cli`, `Command` (enum), `CliCommand` structs
   - Width validation: rejects 0, clamps >200 to 200
   - Converts clap types to internal `CommandKind` enum

2. **`scanner.rs`** - Filesystem traversal
   - `scan_candidates()` - recursively finds JPEGs (.jpg/.jpeg, case-insensitive)
   - Canonicalizes root path, computes stable relative paths
   - Returns `FoundFile { rel_path, abs_path }`

3. **`metadata.rs`** - EXIF extraction
   - `read_capture_date()` - tries EXIF fields in order: DateTimeOriginal → DateTime → DateTimeDigitized
   - `collect_photos()` - combines scanning + EXIF parsing
   - Silently skips files without valid timestamps
   - Returns `PhotoMeta { rel_path, date }`

4. **`analysis.rs`** - Photo analysis algorithms
   - `find_oldest()` / `find_latest()` - with lexicographic path tie-breaking
   - `build_histogram()` - aggregates by year, fills gaps (min..=max)
   - Uses `BTreeMap<i32, usize>` for sorted year counts

5. **`render.rs`** - Output formatting
   - `render_histogram()` - scales bars to width using Unicode block U+2588
   - Guarantees ≥1 block for non-zero counts
   - Zero-count years: no bar, just "YYYY  0"

6. **`error.rs`** - Error handling
   - `PtimeError` enum with thiserror
   - `exit_code()` maps errors: IO/directory → 3, others → 1
   - CLI errors exit 2 (handled by clap)

7. **`lib.rs`** - Integration layer
   - Wires modules together
   - Handles empty results (no output, exit 0)

### Critical Implementation Details

**EXIF Timestamp Fallback:**
The spec requires trying fields in order, but the actual implementation uses:
- `exif::Tag::DateTimeOriginal`
- `exif::Tag::DateTime` (general timestamp)
- `exif::Tag::DateTimeDigitized` (digitization date)

This differs slightly from spec but covers the intent.

**Tie-Breaking Logic:**
When dates match, `find_oldest` and `find_latest` both select the lexicographically smallest path (not largest for latest - this ensures consistent behavior).

**Histogram Gap Filling:**
The histogram includes ALL years between min and max, even with zero photos. This is critical for accurate visualization.

**Path Handling:**
All output uses relative paths from the scan root. The scanner canonicalizes the root once, then strips it from absolute paths to create stable relative paths.

## Testing Strategy

The test suite has 47 tests total:
- **38 unit tests** across all modules
- **9 integration tests** using `assert_cmd`

**Key Test Patterns:**

1. **Empty input handling** - every module tests empty/no-data cases
2. **Edge cases** - width=1, width=200, tie-breaking, zero counts
3. **Integration tests** - test CLI behavior via `Command::cargo_bin()`

**Running Focused Tests:**
```bash
# Test specific functionality
cargo test histogram
cargo test oldest
cargo test exif

# Test with output
cargo test test_name -- --nocapture
```

## Error Handling Rules

1. **IO Errors (exit 3)**: Directory not found, permission denied, file read errors
2. **CLI Errors (exit 2)**: Invalid width (0 or negative), unknown commands (clap handles)
3. **Other Errors (exit 1)**: Unexpected failures
4. **No Photos (exit 0)**: Valid run with empty results - no output

Errors print to stderr via `eprintln!()`, then call `std::process::exit()`.

## Common Gotchas

1. **EXIF Parsing**: Files without EXIF or with malformed dates are silently skipped (not errors)
2. **Width Clamping**: Values >200 are silently clamped; 0 is rejected with error
3. **Relative Paths**: Must canonicalize root before computing relative paths
4. **Date-Only Parsing**: EXIF datetimes are "YYYY:MM:DD HH:MM:SS" but we only use the date portion
5. **Chrono Imports**: Need `use chrono::Datelike` to access `.year()` method

## Adding Features

When extending the codebase:

1. **New subcommands**: Add to `Command` enum in `cli.rs`, add variant to `CommandKind`, handle in `lib.rs` match
2. **New analysis**: Add function to `analysis.rs`, ensure it works with empty input
3. **New EXIF fields**: Modify `field_tags` array in `metadata.rs`
4. **New output formats**: Add rendering function to `render.rs`

Always add corresponding tests before implementing features.
