use anyhow::{anyhow, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
};
use toml;

#[derive(Debug, Serialize, Deserialize)]
pub struct CodeFile {
    pub file_path: String,
    pub extension: String,
    pub size_bytes: u64,
    pub line_count: usize,
    pub has_imports: bool,
    pub has_functions: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectAnalysis {
    pub files: Vec<CodeFile>,
    pub dependency_usage: Vec<DependencyUsage>,
    pub total_use_statements: usize,
    pub project_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DependencyUsage {
    pub name: String,
    pub version: String,
    pub used_lines: usize,   // Number of lines using this library
    pub percentage: f64,     // Percentage of total code
    pub import_count: usize, // Number of import statements (retaining original information)
}
/// Simplified dependency usage for API response
#[derive(Debug, Serialize, Deserialize)]
pub struct LibraryUsage {
    pub name: String,      // Library name
    pub used_lines: usize, // Number of lines using this library
    pub percentage: f64,   // Percentage of total code
}

/// Analyze code files in the specified path
///
/// # Arguments
/// * `relative_path` - Relative path
///
/// # Returns
/// Returns a list of analyzed code file objects
pub fn analyze_code(relative_path: &str) -> Result<ProjectAnalysis> {
    let path = Path::new(relative_path);
    if !path.exists() {
        return Err(anyhow!("Path does not exist: {}", relative_path));
    }

    let mut code_files = Vec::new();
    let mut total_use_statements = 0;
    let mut project_type = "unknown".to_string();
    let mut total_code_lines = 0; // Only counts actual code lines, excluding empty lines and comments

    // Detect project type and get dependencies
    let dependencies = detect_project_and_dependencies(path, &mut project_type)?;
    println!(
        "Detected project type: {}, number of dependencies: {}",
        project_type,
        dependencies.len()
    );

    // Create usage records for each dependency
    let mut dependency_usage_map: HashMap<String, HashMap<String, HashSet<usize>>> = HashMap::new();
    for (name, _) in &dependencies {
        dependency_usage_map.insert(name.clone(), HashMap::new());
    }

    // File total line count mapping
    let mut file_line_counts: HashMap<String, usize> = HashMap::new();

    // Traverse all files in directory
    visit_dirs(path, &mut |entry_path| {
        // Skip directories and non-code files
        if entry_path.is_dir() || !is_code_file(entry_path) {
            return Ok(());
        }

        // Read file content
        let content = match fs::read_to_string(entry_path) {
            Ok(content) => content,
            Err(_) => return Ok(()), // Skip unreadable files
        };

        // Get file extension and relative path
        let extension = entry_path.extension().and_then(|e| e.to_str()).unwrap_or("").to_string();

        let file_path = match entry_path.strip_prefix(path) {
            Ok(p) => p.to_string_lossy().to_string(),
            Err(_) => entry_path.to_string_lossy().to_string(),
        };

        // Analyze file
        let lines: Vec<&str> = content.lines().collect();

        // Count actual code lines (excluding empty lines and comments)
        let code_lines = count_actual_code_lines(&lines, &extension);
        total_code_lines += code_lines;
        file_line_counts.insert(file_path.clone(), code_lines);

        let line_count = lines.len();
        let has_imports = has_import_statements(&content, &extension);
        let has_functions = has_function_definitions(&content, &extension);

        // Count import statements
        let use_count = match extension.as_str() {
            "rs" => count_use_statements(&content),
            _ => 0,
        };
        total_use_statements += use_count;

        // Count usage for each dependency
        for (name, _) in &dependencies {
            // Identify lines using this dependency and store as set to avoid duplicate counting
            let used_lines = identify_dependency_usage_lines(name, &lines, &extension);

            if !used_lines.is_empty() {
                let file_map = dependency_usage_map.get_mut(name).unwrap();
                file_map.insert(file_path.clone(), used_lines.into_iter().collect());
            }
        }

        // Get file size
        let metadata = match fs::metadata(entry_path) {
            Ok(m) => m,
            Err(_) => return Ok(()),
        };
        let size_bytes = metadata.len();

        code_files.push(CodeFile {
            file_path,
            extension,
            size_bytes,
            line_count,
            has_imports,
            has_functions,
        });

        Ok(())
    })?;

    println!("Total project code lines (excluding empty lines and comments): {}", total_code_lines);

    // Build dependency usage
    let mut dependency_usage = Vec::new();
    for (name, version) in dependencies {
        // Calculate total unique lines using this dependency across all files
        let mut total_used_lines = 0;
        let mut import_count = 0;

        if let Some(files_map) = dependency_usage_map.get(&name) {
            for (file_path, lines) in files_map {
                total_used_lines += lines.len();

                // Count import statements (usually at file beginning)
                if let Some(file_code_lines) = file_line_counts.get(file_path) {
                    if *file_code_lines > 0 {
                        for &line_num in lines.iter() {
                            if line_num <= 20 {
                                // Assuming import statements are usually in first 20 lines
                                import_count += 1;
                            }
                        }
                    }
                }
            }
        }

        // Calculate usage percentage based on actual code lines
        let percentage = if total_code_lines > 0 {
            (total_used_lines as f64 / total_code_lines as f64) * 100.0
        } else {
            0.0
        };

        dependency_usage.push(DependencyUsage {
            name,
            version,
            used_lines: total_used_lines,
            percentage,
            import_count,
        });
    }

    // Sort by usage lines
    dependency_usage.sort_by(|a, b| b.used_lines.cmp(&a.used_lines));

    Ok(ProjectAnalysis { files: code_files, dependency_usage, total_use_statements, project_type })
}

/// Recursively traverse directory
fn visit_dirs(dir: &Path, cb: &mut dyn FnMut(&Path) -> Result<()>) -> Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                visit_dirs(&path, cb)?;
            } else {
                cb(&path)?;
            }
        }
    }
    Ok(())
}

/// Find Cargo.lock file in project
fn find_cargo_lock(start_dir: &Path) -> Result<PathBuf> {
    let mut current_dir = start_dir.to_path_buf();

    loop {
        let cargo_lock = current_dir.join("Cargo.lock");
        if cargo_lock.exists() {
            return Ok(cargo_lock);
        }

        if !current_dir.pop() {
            break;
        }
    }

    Err(anyhow!("Could not find Cargo.lock file"))
}

/// Parse Cargo.lock file to get dependency names and versions
fn parse_cargo_lock(lock_path: &Path) -> Result<Vec<(String, String)>> {
    let content = fs::read_to_string(lock_path)?;
    let mut packages = Vec::new();

    // Use toml library to parse Cargo.lock file
    let lock_file: toml::Value = content.parse()?;

    // In Cargo.lock, package info is stored in "package" array
    if let Some(package_array) = lock_file.get("package").and_then(|p| p.as_array()) {
        for package in package_array {
            if let (Some(name), Some(version)) = (
                package.get("name").and_then(|n| n.as_str()),
                package.get("version").and_then(|v| v.as_str()),
            ) {
                packages.push((name.to_string(), version.to_string()));
            }
        }
    }

    Ok(packages)
}

#[allow(dead_code)]
/// Count usage of specified package in content
fn count_package_usage(package_name: &str, content: &str) -> usize {
    let re = Regex::new(&format!(
        r"use\s+{}(?:::|;|\s|$)|\b{}\b::",
        regex::escape(package_name),
        regex::escape(package_name)
    ))
    .unwrap();
    re.find_iter(content).count()
}

/// Count total use statements in content
fn count_use_statements(content: &str) -> usize {
    let re = Regex::new(r"use\s+[^;]+;").unwrap();
    re.find_iter(content).count()
}

/// Check if file is a code file
fn is_code_file(path: &Path) -> bool {
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        let code_extensions = [
            "rs", "cairo", "js", "ts", "py", "go", "c", "cpp", "h", "hpp", "java", "kt", "swift",
            "sol", "move", "sh", "bat", "html", "css", "json", "toml", "yaml", "yml",
        ];

        code_extensions.contains(&ext)
    } else {
        false
    }
}

/// Check if file contains import statements
fn has_import_statements(content: &str, extension: &str) -> bool {
    match extension {
        "rs" => content.contains("use "),
        "py" => content.contains("import ") || content.contains("from "),
        "js" | "ts" => content.contains("import ") || content.contains("require("),
        "cairo" => content.contains("use ") || content.contains("from "),
        "sol" => content.contains("import "),
        _ => false,
    }
}

/// Check if file contains function definitions
fn has_function_definitions(content: &str, extension: &str) -> bool {
    match extension {
        "rs" => content.contains("fn "),
        "py" => content.contains("def "),
        "js" | "ts" => content.contains("function ") || content.contains("=> {"),
        "cairo" => content.contains("fn "),
        "sol" => content.contains("function "),
        _ => false,
    }
}

/// Detect project type and get corresponding dependencies
fn detect_project_and_dependencies(
    path: &Path,
    project_type: &mut String,
) -> Result<Vec<(String, String)>> {
    // Try to detect Rust project
    if let Ok(cargo_lock_path) = find_cargo_lock(path) {
        *project_type = "rust".to_string();
        return parse_cargo_lock(&cargo_lock_path);
    }

    // If no dependency file found, return empty list instead of error
    println!("Warning: No dependency file found, will use empty dependency list");
    Ok(Vec::new())
}

/// Count actual code lines (excluding empty lines and comments)
fn count_actual_code_lines(lines: &[&str], extension: &str) -> usize {
    let mut count = 0;
    let mut in_multi_line_comment = false;

    for line in lines {
        let trimmed = line.trim();

        // Skip empty lines
        if trimmed.is_empty() {
            continue;
        }

        match extension {
            "py" => {
                // Skip Python single-line comments
                if !trimmed.starts_with("#") {
                    count += 1;
                }
            }
            "rs" => {
                // Handle Rust multi-line comments
                if in_multi_line_comment {
                    if trimmed.contains("*/") {
                        in_multi_line_comment = false;
                    }
                    continue;
                }

                // Handle starting multi-line comment
                if trimmed.starts_with("/*") && !trimmed.contains("*/") {
                    in_multi_line_comment = true;
                    continue;
                }

                // Skip single-line comments, but count code+comment lines
                if !(trimmed.starts_with("//") ||
                    trimmed.starts_with("/*") && trimmed.contains("*/"))
                {
                    count += 1;
                }
            }
            "js" | "ts" => {
                // Handle JS/TS multi-line comments
                if in_multi_line_comment {
                    if trimmed.contains("*/") {
                        in_multi_line_comment = false;
                    }
                    continue;
                }

                // Handle starting multi-line comment
                if trimmed.starts_with("/*") && !trimmed.contains("*/") {
                    in_multi_line_comment = true;
                    continue;
                }

                // Skip single-line comments
                if !(trimmed.starts_with("//") ||
                    trimmed.starts_with("/*") && trimmed.contains("*/"))
                {
                    count += 1;
                }
            }
            _ => {
                // For other extensions, simply count non-empty lines
                count += 1;
            }
        }
    }

    count
}

/// Identify line numbers where dependency is used in code
fn identify_dependency_usage_lines(
    package_name: &str,
    lines: &[&str],
    extension: &str,
) -> Vec<usize> {
    let mut used_lines = Vec::new();

    // Lowercase package name for case-insensitive matching
    let package_lower = package_name.to_lowercase();

    match extension {
        "rs" => {
            // First identify imported functions and types
            let mut imported_items: Vec<String> = Vec::new();

            // Match import statements like use demo_dependency_rust::{hello, sum};
            let import_re = Regex::new(&format!(
                r"use\s+{}(?:::|\s*\{{\s*)([^}}]*)(?:\s*}})?",
                regex::escape(package_name)
            ))
            .unwrap();

            for line in lines {
                if let Some(captures) = import_re.captures(line) {
                    if let Some(items_match) = captures.get(1) {
                        let items_str = items_match.as_str();
                        // Split imported items and trim whitespace
                        for item in items_str.split(',') {
                            let clean_item = item.trim().split(' ').next().unwrap_or("").trim();
                            if !clean_item.is_empty() {
                                imported_items.push(clean_item.to_string());
                            }
                        }
                    }
                }
            }

            // Identify four patterns:
            // 1. Direct import: use package_name::<something>
            // 2. Nested import: use <something>::package_name::<something>
            // 3. Path usage: package_name::<something>
            // 4. Macro invocation: package_name!
            let patterns = [
                // Import pattern
                format!(r"^\s*use\s+{}(?:::|;|\s|$)", regex::escape(package_name)),
                format!(r"^\s*use\s+.*::{}", regex::escape(package_name)),
                // Path usage pattern
                format!(r"\b{}\b::", regex::escape(package_name)),
                // Macro invocation pattern
                format!(r"\b{}\!", regex::escape(package_name)),
            ];

            let patterns: Vec<Regex> = patterns.iter().map(|p| Regex::new(p).unwrap()).collect();

            // Create usage patterns for imported items
            let imported_item_patterns: Vec<Regex> = imported_items
                .iter()
                .map(|item| {
                    // Match function calls, struct instantiation or type declarations
                    Regex::new(&format!(r"\b{}\s*(?:\(|\{{|:)", regex::escape(item))).unwrap()
                })
                .collect();

            // Special handling for common Rust libraries
            let special_patterns = match package_lower.as_str() {
                "serde" => vec![
                    Regex::new(r"#\[derive\(.*Serialize.*\)\]").unwrap(),
                    Regex::new(r"#\[derive\(.*Deserialize.*\)\]").unwrap(),
                ],
                "tokio" => vec![
                    Regex::new(r"#\[tokio::main\]").unwrap(),
                    Regex::new(r"tokio::spawn").unwrap(),
                ],
                "anyhow" => vec![Regex::new(r"anyhow!\(").unwrap()],
                "thiserror" => vec![Regex::new(r"#\[derive\(.*Error.*\)\]").unwrap()],
                _ => vec![],
            };

            for (i, line) in lines.iter().enumerate() {
                let mut matched = false;

                // Check standard patterns
                for pattern in &patterns {
                    if pattern.is_match(line) {
                        used_lines.push(i + 1); // Line numbers start at 1
                        matched = true;
                        break;
                    }
                }

                // If no standard pattern matched, check imported item usage
                if !matched && !imported_item_patterns.is_empty() {
                    for pattern in &imported_item_patterns {
                        if pattern.is_match(line) {
                            used_lines.push(i + 1);
                            matched = true;
                            break;
                        }
                    }
                }

                // If special handling exists and still not matched, check special patterns
                if !matched && !special_patterns.is_empty() {
                    for pattern in &special_patterns {
                        if pattern.is_match(line) {
                            used_lines.push(i + 1);
                            break;
                        }
                    }
                }
            }
        }
        "py" => {
            // Match import package or from package import statements
            let import_re = Regex::new(&format!(
                r"^\s*import\s+{}(\s|$|\.|,)|^\s*from\s+{}(\s|$|\.)",
                regex::escape(package_name),
                regex::escape(package_name)
            ))
            .unwrap();

            // Match package usage - more comprehensive detection
            let usage_re = Regex::new(&format!(r"\b{}\.\w+", regex::escape(package_name))).unwrap();

            // Add detection for inline package usage (like pd.DataFrame())
            let inline_re = if package_name == "pandas" {
                Some(Regex::new(r"\bpd\.\w+").unwrap())
            } else if package_name == "numpy" {
                Some(Regex::new(r"\bnp\.\w+").unwrap())
            } else if package_name == "matplotlib" {
                Some(Regex::new(r"\bplt\.\w+").unwrap())
            } else if package_name == "tensorflow" {
                Some(Regex::new(r"\btf\.\w+").unwrap())
            } else if package_name == "pytorch" || package_name == "torch" {
                Some(Regex::new(r"\btorch\.\w+").unwrap())
            } else {
                None
            };

            for (i, line) in lines.iter().enumerate() {
                if import_re.is_match(line) || usage_re.is_match(line) {
                    used_lines.push(i + 1); // Line numbers start at 1
                } else if let Some(re) = &inline_re {
                    if re.is_match(line) {
                        used_lines.push(i + 1);
                    }
                }
            }
        }
        "js" | "ts" => {
            // Match import statements - fix regex syntax
            let import_pattern = format!(
                r#"import.*from\s+['"]{0}['"]|require\(['"]{0}['"]\)"#,
                regex::escape(package_name)
            );
            let import_re = Regex::new(&import_pattern).unwrap();

            // Match package usage - special handling for common libraries
            let usage_pattern = match package_name {
                "react" => r"\bReact\.\w+|useState|useEffect|useContext|useReducer|useCallback|useMemo|useRef".to_string(),
                "lodash" => r"\b_\.\w+".to_string(),
                "jquery" => r"\$\(|\bjQuery\.".to_string(),
                "axios" => r"\baxios\.".to_string(),
                "express" => r"\bexpress\(|\brouter\.".to_string(),
                _ => format!(r"\b{}\.\w+", regex::escape(package_name))
            };
            let usage_re = Regex::new(&usage_pattern).unwrap();

            for (i, line) in lines.iter().enumerate() {
                if import_re.is_match(line) || usage_re.is_match(line) {
                    used_lines.push(i + 1);
                }
            }
        }
        _ => {}
    }

    used_lines
}

#[allow(dead_code)]
/// Count import statements in JavaScript/TypeScript files
fn count_js_imports(content: &str) -> usize {
    let import_re = Regex::new(r"^\s*(import|require)\b").unwrap();
    content.lines().filter(|line| import_re.is_match(line)).count()
}

/// Convert ProjectAnalysis to simplified library usage list
pub fn to_library_usage(analysis: &ProjectAnalysis) -> Vec<LibraryUsage> {
    analysis
        .dependency_usage
        .iter()
        .map(|dep| LibraryUsage {
            name: dep.name.clone(),
            used_lines: dep.used_lines,
            percentage: dep.percentage,
        })
        .collect()
}
