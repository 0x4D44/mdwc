use std::collections::HashSet;
use std::error::Error;
use std::fs;
use std::io::Read;
use std::path::Path;

use colored::*;
use glob::glob;
use pdf_extract::extract_text;
use regex::Regex;
use zip::ZipArchive;

const FILENAME_WIDTH: usize = 45; // Maximum width for the file name column

#[derive(Debug)]
pub struct WordCount {
    pub file_path: String,
    pub unique_words: usize,
    pub total_words: usize,
}

/// Extracts the content of a file. For PDFs it uses `pdf_extract`, for DOCX files it reads the internal
/// XML and strips out tags, and all other files are read as plain text.
fn extract_file_content(file_path: &str) -> Result<String, Box<dyn Error>> {
    let path = Path::new(file_path);
    match path.extension().and_then(|ext| ext.to_str()) {
        Some("pdf") => {
            let content = extract_text(file_path)?;
            Ok(content)
        }
        Some("docx") => {
            let content = extract_docx_text(file_path)?;
            Ok(content)
        }
        _ => {
            // Default to regular text file handling
            Ok(fs::read_to_string(file_path)?)
        }
    }
}

/// Extracts text from a DOCX file by opening it as a ZIP archive,
/// reading the "word/document.xml" file, and then removing XML tags.
fn extract_docx_text(file_path: &str) -> Result<String, Box<dyn Error>> {
    let file = fs::File::open(file_path)?;
    let mut archive = ZipArchive::new(file)?;
    let mut document = archive.by_name("word/document.xml")?;
    let mut xml_content = String::new();
    document.read_to_string(&mut xml_content)?;

    // A simple regex to remove XML tags.
    let re = Regex::new(r"<[^>]+>")?;
    let text = re.replace_all(&xml_content, " ");
    Ok(text.into_owned())
}

/// Counts words in the file, returning a `WordCount` structure.
pub fn count_words_in_file(file_path: &str) -> Result<WordCount, Box<dyn Error>> {
    let contents = extract_file_content(file_path)?;
    let words: Vec<String> = contents
        .split(|c: char| !c.is_alphabetic())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_lowercase())
        .collect();

    let unique_words = words.iter().collect::<HashSet<_>>().len();

    Ok(WordCount {
        file_path: file_path.to_string(),
        unique_words,
        total_words: words.len(),
    })
}

/// Processes files matching the given glob pattern.
pub fn process_files(pattern: &str) -> Result<Vec<WordCount>, Box<dyn Error>> {
    let mut results = Vec::new();
    
    for entry in glob(pattern)? {
        match entry {
            Ok(path) => {
                if path.is_file() {
                    // Skip temporary Word files (start with ~$)
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        if name.starts_with("~$") {
                            continue;
                        }
                    }
                    match count_words_in_file(path.to_str().unwrap()) {
                        Ok(count) => results.push(count),
                        Err(e) => eprintln!("Error processing {}: {}", path.display(), e),
                    }
                }
            }
            Err(e) => eprintln!("Glob error: {}", e),
        }
    }

    if results.is_empty() {
        return Err("No files found matching the pattern".into());
    }

    Ok(results)
}

/// Formats a number with commas.
fn format_number(num: usize) -> String {
    num.to_string()
        .chars()
        .rev()
        .collect::<Vec<_>>()
        .chunks(3)
        .map(|chunk| chunk.iter().collect::<String>())
        .collect::<Vec<_>>()
        .join(",")
        .chars()
        .rev()
        .collect()
}

/// Truncates a file name if it exceeds `max_len` characters and appends an ellipsis.
fn format_filename(name: &str, max_len: usize) -> String {
    if name.chars().count() > max_len {
        // Reserve space for the ellipsis ("...")
        let truncated: String = name.chars().take(max_len.saturating_sub(3)).collect();
        format!("{}...", truncated)
    } else {
        name.to_string()
    }
}

pub fn run(args: &[String], writer: &mut impl std::io::Write) -> Result<(), Box<dyn Error>> {
    if args.len() < 1 {
        // This case essentially shouldn't happen with std::env::args() usually having at least 1 (the binary name),
        // but if we pass a slice of args excluding binary name, we might see 0.
        // Let's assume input args are [pattern1, pattern2...] (excluding binary name) for the logic loop,
        // OR we stick to the convention that args[0] is binary name.
        // The original code used args[1..], so let's stick to receiving the full args vector.
        return Err("Not enough arguments".into());
    }
    
    // Check if we have patterns (i.e. length >= 2 if args[0] is binary)
    if args.len() < 2 {
        writeln!(writer, "Usage: {} <file_pattern> [file_pattern...]", args[0])?;
        writeln!(writer, "Supported file types: .txt, .pdf, .docx")?;
        writeln!(writer, "Examples:")?;
        writeln!(writer, "  {} *.txt", args[0])?;
        writeln!(writer, "  {} *.pdf", args[0])?;
        writeln!(writer, "  {} *.docx", args[0])?;
        writeln!(writer, "  {} docs/*.{{txt,pdf,docx}}", args[0])?;
        return Err("Invalid usage".into());
    }

    let mut grand_total_words = 0;
    let mut grand_total_unique = HashSet::new();
    let mut files_processed = 0;

    for pattern in &args[1..] {
        match process_files(pattern) {
            Ok(results) => {
                writeln!(writer, "\n{} '{}':",
                    "Analysis for files matching pattern".blue().bold(),
                    pattern.yellow())?;
                writeln!(writer, "{}", "-".repeat(80).dimmed())?;
                
                let mut pattern_total_words = 0;
                let mut pattern_unique_words = HashSet::new();

                // Process each file's results
                for result in results {
                    pattern_total_words += result.total_words;
                    
                    // Extract file contents again to update unique words accurately.
                    if let Ok(contents) = extract_file_content(&result.file_path) {
                        let words: Vec<String> = contents
                            .split(|c: char| !c.is_alphabetic())
                            .filter(|s| !s.is_empty())
                            .map(|s| s.to_lowercase())
                            .collect();
                        pattern_unique_words.extend(words.clone());
                        grand_total_unique.extend(words);
                    }
                    
                    // Extract just the file name from the full path.
                    let raw_name = Path::new(&result.file_path)
                        .file_name()
                        .and_then(|s| s.to_str())
                        .unwrap_or(&result.file_path);
                    let display_name = format_filename(raw_name, FILENAME_WIDTH);
                    
                    // Print file results using fixed-width formatting.
                    writeln!(writer,
                        "{:<width$}: {:>10} {} {:>10} {}",
                        display_name,
                        format_number(result.unique_words).cyan(),
                        "unique words out of".dimmed(),
                        format_number(result.total_words).cyan(),
                        "total words".dimmed(),
                        width = FILENAME_WIDTH
                    )?;
                    
                    files_processed += 1;
                }

                // Print pattern summary.
                writeln!(writer, "{}", "-".repeat(80).dimmed())?;
                writeln!(writer,
                    "{} {:>10} {} {:>10} {}\n",
                    "Summary for pattern:".blue().bold(),
                    format_number(pattern_unique_words.len()).bright_cyan(),
                    "unique words out of".dimmed(),
                    format_number(pattern_total_words).bright_cyan(),
                    "total words".dimmed()
                )?;

                grand_total_words += pattern_total_words;
            }
            Err(e) => writeln!(writer, "{} processing pattern '{}': {}",
                "Error".red().bold(), pattern.yellow(), e)?,
        }
    }

    // Print grand total if we processed at least one file.
    if files_processed > 0 {
        writeln!(writer, "{}", "=".repeat(80).blue())?;
        writeln!(writer,
            "{} ({} files processed):",
            "GRAND TOTAL".blue().bold(),
            format_number(files_processed).bright_yellow()
        )?;
        let ratio = (grand_total_unique.len() as f64 / grand_total_words as f64) * 100.0;
        writeln!(writer,
            "{} {:>10}\n{} {:>10}\n{} {}",
            "Total unique words:".dimmed(),
            format_number(grand_total_unique.len()).bright_cyan(),
            "Total words:       ".dimmed(),
            format_number(grand_total_words).bright_cyan(),
            "Unique ratio:      ".dimmed(),
            format!("{:>9.1}%", ratio).green()
        )?;
        writeln!(writer, "{}", "=".repeat(80).blue())?;
    }
    
    Ok(())
}

fn main() {
    // Print version header
    println!(
        "{} {}",
        env!("CARGO_PKG_NAME").bright_cyan().bold(),
        format!("v{}", env!("CARGO_PKG_VERSION")).bright_yellow()
    );

    let args: Vec<String> = std::env::args().collect();
    if let Err(_) = run(&args, &mut std::io::stdout()) {
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::TempDir;
    use zip::write::FileOptions;

    fn create_test_file(dir: &TempDir, filename: &str, content: &str) -> String {
        let file_path = dir.path().join(filename);
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "{}", content).unwrap();
        file_path.to_str().unwrap().to_string()
    }

    fn create_docx_file(dir: &TempDir, filename: &str, content: &str) -> String {
        let file_path = dir.path().join(filename);
        let file = File::create(&file_path).unwrap();
        let mut zip = zip::ZipWriter::new(file);

        let options = FileOptions::default().compression_method(zip::CompressionMethod::Stored);
        zip.start_file("word/document.xml", options).unwrap();
        
        // Wrap content in minimal XML
        let xml = format!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>
            <w:document xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\">
            <w:body><w:p><w:r><w:t>{}</w:t></w:r></w:p></w:body></w:document>",
            content
        );
        zip.write_all(xml.as_bytes()).unwrap();
        zip.finish().unwrap();

        file_path.to_str().unwrap().to_string()
    }

    #[test]
    fn test_empty_file() {
        let dir = TempDir::new().unwrap();
        let file_path = create_test_file(&dir, "empty.txt", "");
        let result = count_words_in_file(&file_path).unwrap();
        assert_eq!(result.unique_words, 0);
        assert_eq!(result.total_words, 0);
    }

    #[test]
    fn test_single_word() {
        let dir = TempDir::new().unwrap();
        let file_path = create_test_file(&dir, "single.txt", "hello");
        let result = count_words_in_file(&file_path).unwrap();
        assert_eq!(result.unique_words, 1);
        assert_eq!(result.total_words, 1);
    }

    #[test]
    fn test_repeated_words() {
        let dir = TempDir::new().unwrap();
        let file_path = create_test_file(&dir, "repeated.txt", "hello hello HELLO");
        let result = count_words_in_file(&file_path).unwrap();
        assert_eq!(result.unique_words, 1);
        assert_eq!(result.total_words, 3);
    }

    #[test]
    fn test_multiple_words() {
        let dir = TempDir::new().unwrap();
        let file_path = create_test_file(&dir, "multiple.txt", "The quick brown fox jumps");
        let result = count_words_in_file(&file_path).unwrap();
        assert_eq!(result.unique_words, 5);
        assert_eq!(result.total_words, 5);
    }

    #[test]
    fn test_punctuation() {
        let dir = TempDir::new().unwrap();
        let file_path = create_test_file(&dir, "punct.txt", "hello, world! How are you?");
        let result = count_words_in_file(&file_path).unwrap();
        assert_eq!(result.unique_words, 5);
        assert_eq!(result.total_words, 5);
    }

    #[test]
    fn test_glob_pattern() {
        let dir = TempDir::new().unwrap();
        create_test_file(&dir, "test1.txt", "hello world");
        create_test_file(&dir, "test2.txt", "hello rust");
        
        let pattern = format!("{}/*.txt", dir.path().to_str().unwrap());
        let results = process_files(&pattern).unwrap();
        
        assert_eq!(results.len(), 2);
        // Both files contain 2 words each.
        assert!(results.iter().all(|r| r.unique_words == 2));
    }

    #[test]
    fn test_nonexistent_pattern() {
        let result = process_files("nonexistent*.txt");
        assert!(result.is_err());
    }

    // New test to check the aggregated total words across multiple files.
    #[test]
    fn test_aggregation_totals() {
        let dir = TempDir::new().unwrap();
        // Create two files with known content:
        // file1.txt: "hello world" (2 words)
        // file2.txt: "rust language" (2 words)
        create_test_file(&dir, "file1.txt", "hello world");
        create_test_file(&dir, "file2.txt", "rust language");

        let pattern = format!("{}/*.txt", dir.path().to_str().unwrap());
        let results = process_files(&pattern).unwrap();

        // Expected total words: 2 + 2 = 4
        let expected_total_words = 4;
        let actual_total_words: usize = results.iter().map(|r| r.total_words).sum();
        assert_eq!(
            actual_total_words, 
            expected_total_words,
            "Aggregated total words should equal the sum of words in each file"
        );
    }

    #[test]
    fn test_format_number() {
        assert_eq!(format_number(0), "0");
        assert_eq!(format_number(10), "10");
        assert_eq!(format_number(100), "100");
        assert_eq!(format_number(1000), "1,000");
        assert_eq!(format_number(1000000), "1,000,000");
        assert_eq!(format_number(123456789), "123,456,789");
    }

    #[test]
    fn test_format_filename() {
        assert_eq!(format_filename("short.txt", 10), "short.txt");
        assert_eq!(format_filename("exactsize.txt", 13), "exactsize.txt");
        assert_eq!(format_filename("longerfilename.txt", 10), "longerf...");
        // Check edge case where max_len is very small
        assert_eq!(format_filename("abcd", 3), "..."); 
    }

    #[test]
    fn test_docx_extraction() {
        let dir = TempDir::new().unwrap();
        let file_path = create_docx_file(&dir, "test.docx", "Hello Docx World");
        let result = count_words_in_file(&file_path).unwrap();
        
        assert_eq!(result.unique_words, 3);
        assert_eq!(result.total_words, 3);
    }

    #[test]
    fn test_run_usage() {
        let args = vec!["mdwc".to_string()]; // No patterns provided
        let mut buffer = Vec::new();
        
        let result = run(&args, &mut buffer);
        assert!(result.is_err()); // Should return "Invalid usage" or similar error
        
        let output = String::from_utf8(buffer).unwrap();
        assert!(output.contains("Usage:"));
        assert!(output.contains("Supported file types:"));
    }

    #[test]
    fn test_run_file_processing() {
        let dir = TempDir::new().unwrap();
        create_test_file(&dir, "run_test.txt", "hello run world");
        
        let pattern = format!("{}/*.txt", dir.path().to_str().unwrap());
        let args = vec!["mdwc".to_string(), pattern];
        let mut buffer = Vec::new();

        let result = run(&args, &mut buffer);
        assert!(result.is_ok());

        let output = String::from_utf8(buffer).unwrap();
        assert!(output.contains("Analysis for files matching pattern"));
        assert!(output.contains("run_test.txt"));
        assert!(output.contains("3 unique words out of          3 total words"));
        assert!(output.contains("GRAND TOTAL"));
    }

    #[test]
    fn test_run_no_matching_files() {
        let dir = TempDir::new().unwrap();
        // Create no files
        let pattern = format!("{}/*.txt", dir.path().to_str().unwrap());
        let args = vec!["mdwc".to_string(), pattern];
        let mut buffer = Vec::new();

        let result = run(&args, &mut buffer);
        assert!(result.is_ok()); // Should be ok, just prints error per pattern

        let output = String::from_utf8(buffer).unwrap();
        // Should contain the error for the pattern
        assert!(output.contains("Error processing pattern"));
        // Should NOT contain GRAND TOTAL
        assert!(!output.contains("GRAND TOTAL"));
    }

    #[test]
    fn test_pdf_branch_coverage() {
        let dir = TempDir::new().unwrap();
        // Create a dummy PDF file (invalid content)
        // This won't successfully extract text, but it will enter the "pdf" match arm
        // and likely return an Err from extract_text.
        let file_path = create_test_file(&dir, "invalid.pdf", "not a real pdf");
        
        let result = count_words_in_file(&file_path);
        // We expect an error because it's not a valid PDF
        assert!(result.is_err());
    }

    #[test]
    fn test_process_invalid_pdf_integration() {
        let dir = TempDir::new().unwrap();
        create_test_file(&dir, "bad.pdf", "invalid pdf content");
        
        let pattern = format!("{}/*.pdf", dir.path().to_str().unwrap());
        let args = vec!["mdwc".to_string(), pattern];
        let mut buffer = Vec::new();

        // This will find the file, try to process it, fail at extraction,
        // and print to stderr (which we don't capture here, but we execute the path).
        // The run function itself should return Ok because it handled the error gracefully.
        let result = run(&args, &mut buffer);
        assert!(result.is_ok());
        
        let output = String::from_utf8(buffer).unwrap();
        // Since the error is printed to stderr in process_files (via eprintln!),
        // and run() only prints to buffer on success of processing files,
        // we might not see the file in the success list.
        assert!(!output.contains("bad.pdf")); 
        
        // However, we verify that the Summary line is still printed (even if 0 files success)
        // OR if the list was empty of successes, maybe it behaves differently.
        // Actually, if results is empty (all failed), process_files returns Err("No files found...")
        // Wait, process_files loop: if error occurs, it prints eprintln and continues.
        // If ALL files fail, results is empty. process_files returns Err.
        // So run() receives Err.
        
        // Let's check process_files logic again.
        // for entry in glob...
        //    if path.is_file() 
        //       match count_words_in_file...
        //          Ok -> results.push
        //          Err -> eprintln (Line 84)
        // if results.is_empty() -> Err("No files found...")
        
        // So if we only have 1 bad file, results is empty, so run() gets Err.
        // Let's include one GOOD file too, so process_files returns Ok, but still hits the error path for the bad one.
        create_test_file(&dir, "good.txt", "hello");
        let pattern_all = format!("{}/*.*", dir.path().to_str().unwrap());
        let args_all = vec!["mdwc".to_string(), pattern_all];
        
        let mut buffer2 = Vec::new(); // Use a new buffer
        let result_all = run(&args_all, &mut buffer2);
        // Now we should have 1 success, so process_files returns Ok.
        assert!(result_all.is_ok());
    }
}
