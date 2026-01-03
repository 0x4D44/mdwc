# Code Coverage Improvement Plan

## Goal
Increase code coverage from ~40% to >98% by systematically addressing untested areas.

## Stages

### Stage 1: Helper Functions & Formatting
**Objective:** Cover utility functions used for display.
*   **Action:** Add unit tests for `format_number` and `format_filename`.
*   **Target:** 100% coverage of lines 100-123.

### Stage 2: DOCX Support
**Objective:** Verify DOCX text extraction logic.
*   **Action:**
    *   Add a test helper to create a valid `.docx` file (which is a ZIP containing `word/document.xml`).
    *   Add a test case `test_docx_extraction` that generates this file and verifies `count_words_in_file` correctly extracts and counts words from it.
*   **Target:** Coverage of `extract_docx_text` and the docx branch in `extract_file_content`.

### Stage 3: PDF Support (Architecture/Mocking)
**Objective:** Cover the PDF extraction branch.
*   **Action:**
    *   Since generating a valid PDF in tests is complex without heavy dependencies, refactor `extract_file_content` to use a trait-based strategy or simple dependency injection if needed.
    *   *Alternative:* For this project, a simpler approach is to use conditional compilation or just acknowledge the external dependency. However, to hit strict 98%, we might need to mock the `extract_text` call.
    *   *Plan:* Refactor extraction into a `TextExtractor` trait. Implement `RealExtractor` (uses pdf-extract/zip) and `MockExtractor`.
    *   *Simpler Plan:* Since `pdf-extract` is a direct dependency, we can't mock it easily without changing the architecture. We will attempt to rely on the `pdf` feature tests if we can add a tiny valid PDF to a `tests/fixtures` folder, or strictly refactor to decouple the *call* to the library.
    *   *Decision for Stage 3:* Refactor `extract_file_content` to allow dependency injection of the extraction logic? No, that's over-engineering.
    *   *Revised Stage 3:* Create a very minimal valid PDF byte array in the test file (hardcoded) and write it to disk to test the integration. If that's too binary-heavy, we will accept the gap or try to mock.
    *   *Let's try:* Hardcode a minimal PDF header/content if possible, or just focus on the other 95% first.

### Stage 4: CLI Integration (Refactoring `main`)
**Objective:** Cover the application entry point, argument parsing, and output formatting.
*   **Action:**
    *   Refactor `main` logic into a public `run(args: &[String], writer: &mut impl Write) -> Result<(), Box<dyn Error>>` function.
    *   Update `main` to simply call `run`.
    *   Add "integration" style tests that call `run` with various arguments (valid patterns, invalid patterns, help flags) and assert on the output captured in the writer.
*   **Target:** Coverage of lines 125-214 (the previous `main` body).

### Stage 5: Error Handling
**Objective:** Cover edge cases.
*   **Action:** Add tests for:
    *   Files that exist but are unreadable (permissions).
    *   Corrupt DOCX files (invalid zip).
