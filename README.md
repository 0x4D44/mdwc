# mdwc

**mdwc** (Markdown/Document Word Count) is a fast and flexible command-line tool written in Rust for analyzing word usage across various document formats. It provides both total word counts and unique word counts, helping writers and developers track text complexity and volume.

## Features

-   **Multi-Format Support**: Native support for:
    -   Plain Text (`.txt`)
    -   PDF Documents (`.pdf`)
    -   Microsoft Word Documents (`.docx`)
-   **Batch Processing**: Accepts glob patterns to analyze multiple files or entire directories at once (e.g., `*.txt`, `docs/**/*.pdf`).
-   **Deep Analysis**: Calculates both **total word count** and **unique word count** for each file.
-   **Aggregated Statistics**: Provides a summary per file pattern and a grand total across all processed files.
-   **Performance**: Built with Rust for speed and safety.

## Installation

### Prerequisites

You need the Rust toolchain installed on your machine. If you haven't installed it yet, visit [rustup.rs](https://rustup.rs/).

### Building from Source

1.  Clone the repository:
    ```bash
    git clone https://github.com/yourusername/mdwc.git
    cd mdwc
    ```

2.  Build the project using Cargo:
    ```bash
    cargo build --release
    ```

The compiled binary will be located in `target/release/mdwc`.

## Usage

Run the tool by providing one or more file patterns as arguments.

```bash
# Using cargo to run directly
cargo run --release -- <pattern1> [pattern2] ...

# Or using the binary directly (if added to PATH or from target/release)
./mdwc <pattern1> [pattern2] ...
```

### Examples

**Analyze a single text file:**
```bash
mdwc notes.txt
```

**Analyze all PDF files in the current directory:**
```bash
mdwc "*.pdf"
```

**Analyze multiple patterns (e.g., all docs in a folder):**
```bash
mdwc "chapters/*.docx" "references/*.pdf"
```

> **Note on Glob Patterns:** When using wildcards like `*`, it is recommended to wrap the pattern in quotes (e.g., `"*.txt"`) to prevent your shell from expanding them before `mdwc` receives them. `mdwc` handles the expansion internally to ensure consistent behavior across operating systems.

## Sample Output

```text
Analysis for files matching pattern '*.txt':
--------------------------------------------------------------------------------
notes.txt                                    :        120 unique words out of        450 total words
draft.txt                                    :        340 unique words out of      1,200 total words
--------------------------------------------------------------------------------
Summary for pattern:        410 unique words out of      1,650 total words

Analysis for files matching pattern '*.pdf':
--------------------------------------------------------------------------------
specs.pdf                                    :        890 unique words out of      5,000 total words
--------------------------------------------------------------------------------
Summary for pattern:        890 unique words out of      5,000 total words

================================================================================
GRAND TOTAL (3 files processed):
Total unique words:      1,150
Total words:             6,650
Unique ratio:             17.3%
================================================================================
```

## Development

### Running Tests

The project includes a suite of unit tests to verify file parsing and counting logic.

```bash
cargo test
```

### Project Structure

-   `src/main.rs`: Contains the entry point and all core logic, including:
    -   File format extraction (PDF, DOCX, Text).
    -   Word tokenization and counting.
    -   CLI argument parsing and output formatting.

## Dependencies

-   [`glob`](https://crates.io/crates/glob): File pattern matching.
-   [`pdf-extract`](https://crates.io/crates/pdf-extract): Extraction of text from PDF files.
-   [`zip`](https://crates.io/crates/zip) & [`regex`](https://crates.io/crates/regex): Used for parsing `.docx` files (XML content).
