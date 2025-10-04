# ptime

A command-line tool for analyzing photo timestamps from JPEG files.

## Features

- **Find oldest photo**: Locate the earliest photo in a directory tree based on EXIF timestamps
- **Find latest photo**: Locate the most recent photo in a directory tree
- **Histogram visualization**: Display a year-by-year histogram of photos with ASCII bar charts
- **EXIF fallback parsing**: Automatically tries multiple EXIF date fields (DateTimeOriginal, DateTime, DateTimeDigitized)
- **Recursive scanning**: Walks entire directory trees to find all JPEG files
- **Case-insensitive**: Recognizes `.jpg`, `.jpeg`, `.JPG`, `.JPEG` extensions

## Installation

### Build from source

```bash
cargo build --release
```

The binary will be available at `./target/release/ptime`.

### Install locally

```bash
cargo install --path .
```

## Usage

### Find the oldest photo

```bash
ptime oldest [DIRECTORY]
```

Example output:
```
photos/vacation/IMG_1234.jpg 2019-07-15
```

### Find the most recent photo

```bash
ptime latest [DIRECTORY]
```

Example output:
```
photos/recent/IMG_9876.jpg 2024-12-25
```

### Show histogram of photos by year

```bash
ptime hist [--width WIDTH] [DIRECTORY]
```

Example output:
```
2019 ████████████████████ 45
2020 ████████████ 28
2021 ███████ 15
2022 ████████████████████████████ 62
2023 ██████████████████████████████████████████████████ 112
2024 ████████ 18
```

Options:
- `--width, -w`: Width of histogram bars (1-200, default: 50)
- `DIRECTORY`: Directory to scan (default: current directory `.`)

## How it works

1. **Scanning**: Recursively walks the directory tree to find all JPEG files (`.jpg` and `.jpeg` extensions, case-insensitive)
2. **EXIF parsing**: Reads EXIF metadata from each JPEG, trying timestamps in this order:
   - `DateTimeOriginal` (when photo was taken)
   - `DateTime` (general timestamp)
   - `DateTimeDigitized` (when photo was digitized)
3. **Analysis**:
   - For oldest/latest: Finds the photo with earliest/latest date (lexicographic path tie-breaking)
   - For histogram: Groups photos by year and fills gaps between min/max years
4. **Output**: Displays results in the specified format

## Exit Codes

- `0`: Success
- `1`: General error
- `2`: Usage error (e.g., invalid arguments)
- `3`: IO error (e.g., directory not found, permission denied)

## Examples

### Analyze current directory

```bash
ptime oldest
ptime latest
ptime hist
```

### Analyze specific directory

```bash
ptime oldest ~/Pictures
ptime hist --width 100 ~/Photos/Vacation
```

### Photos with no EXIF timestamps

Files without valid EXIF timestamps are silently skipped. If no photos have timestamps:

```bash
ptime oldest /path/to/photos
# (no output)
```

## Testing

Run the test suite:

```bash
cargo test
```

This runs:
- 38 unit tests (CLI parsing, scanning, EXIF parsing, analysis, rendering)
- 9 integration tests (end-to-end CLI behavior)

Run with verbose output:

```bash
cargo test -- --nocapture
```

Format and lint:

```bash
cargo fmt
cargo clippy -- -D warnings
```

## Architecture

The project is organized into modules:

- `cli`: Command-line argument parsing with clap
- `scanner`: Filesystem traversal and JPEG discovery
- `metadata`: EXIF extraction and date parsing
- `analysis`: Photo analysis (oldest, latest, histogram)
- `render`: ASCII histogram rendering
- `error`: Error types and exit code mapping

## Limitations

- Only processes JPEG files (`.jpg`, `.jpeg`)
- Requires valid EXIF metadata with date/time fields
- Files without EXIF timestamps are skipped
- No support for other image formats (PNG, TIFF, RAW, etc.)
- Histogram shows years only (not months or days)

## Future Enhancements

Potential improvements for future versions:

- Support for additional image formats (PNG, TIFF, HEIF)
- More granular histogram options (by month, by day)
- JSON/CSV output formats
- Parallel processing for large directories
- Configurable EXIF field priority
- Timezone handling
- Progress indicators for large scans

## License

MIT.

## Development

The implementation follows a structured approach documented in `PROMPTS.md`:

1. Project scaffolding and dependencies
2. CLI parsing and validation
3. Filesystem scanning
4. EXIF metadata extraction
5. Analysis algorithms
6. Histogram rendering
7. Integration and testing

Each phase has corresponding commits in the Git history.
