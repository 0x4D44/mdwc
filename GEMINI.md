# mdwc (Markdown/Document Word Count)

## Project Overview
`mdwc` is a Rust-based Command Line Interface (CLI) tool designed to count total and unique words across multiple file formats. It supports processing single files or batch processing via glob patterns.

**Key Features:**
*   **Multi-format Support:** handles `.txt` (plain text), `.pdf`, and `.docx` files.
*   **Batch Processing:** Supports glob patterns (e.g., `*.txt`, `docs/*.{pdf,docx}`) to analyze multiple files at once.
*   **Analysis Metrics:** Reports total word count and unique word count per file.
*   **Aggregated Statistics:** Provides summaries per glob pattern and a grand total across all processed files.

## Architecture
The project is a standard Rust binary application contained primarily within `src/main.rs`.

*   **`main.rs`**: Contains the entry point and all core logic.
    *   **Extraction:** `extract_file_content` dispatches to specific handlers (`extract_docx_text` for .docx, `pdf_extract` crate for .pdf).
    *   **Analysis:** `count_words_in_file` normalizes text (lowercase) and tokenizes based on non-alphabetic characters.
    *   **Execution:** `process_files` handles glob expansion and file iteration.
*   **Dependencies:**
    *   `glob`: For file pattern matching.
    *   `pdf-extract`: For parsing PDF content.
    *   `zip` & `regex`: For parsing DOCX content (treating it as a zipped XML structure).

## Building and Running

### Prerequisites
*   Rust toolchain (Cargo, rustc)

### Build
To build the project in release mode:
```bash
cargo build --release
```

### Run
To run the tool directly via Cargo:
```bash
cargo run -- <file_pattern> [file_pattern...]
```

**Examples:**
```bash
# Analyze all text files in the current directory
cargo run -- "*.txt"

# Analyze specific formats in a docs folder
cargo run -- "docs/*.pdf" "docs/*.docx"
```

### Tests
The project includes unit tests for core logic (counting, globbing, edge cases).
```bash
cargo test
```

## Development Conventions
*   **Code Style:** Standard Rust formatting (`cargo fmt`).
*   **Error Handling:** Uses `Box<dyn Error>` for flexible error propagation in the CLI context.
*   **Testing:** Unit tests are co-located in `src/main.rs` under the `tests` module.
