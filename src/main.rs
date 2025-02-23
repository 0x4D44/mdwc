use std::collections::HashSet;
use std::error::Error;
use std::fs;
use std::io::Read;
use std::path::Path;

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

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <file_pattern> [file_pattern...]", args[0]);
        eprintln!("Supported file types: .txt, .pdf, .docx");
        eprintln!("Examples:");
        eprintln!("  {} *.txt", args[0]);
        eprintln!("  {} *.pdf", args[0]);
        eprintln!("  {} *.docx", args[0]);
        eprintln!("  {} docs/*.{{txt,pdf,docx}}", args[0]);
        std::process::exit(1);
    }

    let mut grand_total_words = 0;
    let mut grand_total_unique = HashSet::new();
    let mut files_processed = 0;

    for pattern in &args[1..] {
        match process_files(pattern) {
            Ok(results) => {
                println!("\nAnalysis for files matching pattern '{}':", pattern);
                println!("{:-<80}", "");  // Print a separator line
                
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
                    println!(
                        "{:<width$}: {:>10} unique words out of {:>10} total words",
                        display_name,
                        format_number(result.unique_words),
                        format_number(result.total_words),
                        width = FILENAME_WIDTH
                    );
                    
                    files_processed += 1;
                }

                // Print pattern summary.
                println!("{:-<80}", "");  // Separator line
                println!(
                    "Summary for pattern: {:>10} unique words out of {:>10} total words\n",
                    format_number(pattern_unique_words.len()),
                    format_number(pattern_total_words)
                );

                grand_total_words += pattern_total_words;
            }
            Err(e) => eprintln!("Error processing pattern '{}': {}", pattern, e),
        }
    }

    // Print grand total if we processed at least one file.
    if files_processed > 0 {
        println!("{:=<80}", "");  // Double separator line
        println!(
            "GRAND TOTAL ({} files processed):", 
            format_number(files_processed)
        );
        println!(
            "Total unique words: {:>10}\nTotal words:       {:>10}\nUnique ratio:      {:>9.1}%",
            format_number(grand_total_unique.len()),
            format_number(grand_total_words),
            (grand_total_unique.len() as f64 / grand_total_words as f64) * 100.0
        );
        println!("{:=<80}", "");  // Double separator line
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::TempDir;

    fn create_test_file(dir: &TempDir, filename: &str, content: &str) -> String {
        let file_path = dir.path().join(filename);
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "{}", content).unwrap();
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
}
