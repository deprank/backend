use std::path::{Path, PathBuf};
use std::fs;
use std::collections::{HashMap, HashSet};
use regex::Regex;
use anyhow::{Result, anyhow};
use serde::{Serialize, Deserialize};
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
    pub used_lines: usize,     // 使用该库的代码行数
    pub percentage: f64,       // 占总代码的百分比
    pub import_count: usize,   // 导入语句数量（保留原有信息）
}
/// 简化版的依赖使用情况，用于API返回
#[derive(Debug, Serialize, Deserialize)]
pub struct LibraryUsage {
    pub name: String,          // 库名称
    pub used_lines: usize,     // 使用该库的代码行数
    pub percentage: f64,       // 占总代码的百分比
}

/// 分析指定路径下的代码文件
/// 
/// # 参数
/// * `relative_path` - 相对路径
/// 
/// # 返回
/// 返回分析的代码文件对象列表
pub fn analyze_code(relative_path: &str) -> Result<ProjectAnalysis> {
    let path = Path::new(relative_path);
    if !path.exists() {
        return Err(anyhow!("Path does not exist: {}", relative_path));
    }
    
    let mut code_files = Vec::new();
    let mut total_use_statements = 0;
    let mut project_type = "unknown".to_string();
    let mut total_code_lines = 0; // 只计算实际代码行，不包括空行和注释
    
    // 检测项目类型并获取依赖
    let dependencies = detect_project_and_dependencies(path, &mut project_type)?;
    println!("检测到项目类型: {}, 依赖包数量: {}", project_type, dependencies.len());
    
    // 创建每个依赖的使用记录
    let mut dependency_usage_map: HashMap<String, HashMap<String, HashSet<usize>>> = HashMap::new();
    for (name, _) in &dependencies {
        dependency_usage_map.insert(name.clone(), HashMap::new());
    }
    
    // 文件总行数映射
    let mut file_line_counts: HashMap<String, usize> = HashMap::new();
    
    // 遍历目录中的所有文件
    visit_dirs(path, &mut |entry_path| {
        // 跳过目录和非代码文件
        if entry_path.is_dir() || !is_code_file(entry_path) {
            return Ok(());
        }
        
        // 读取文件内容
        let content = match fs::read_to_string(entry_path) {
            Ok(content) => content,
            Err(_) => return Ok(()), // 跳过无法读取的文件
        };
        
        // 获取文件扩展名和相对路径
        let extension = entry_path.extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_string();
        
        let file_path = match entry_path.strip_prefix(path) {
            Ok(p) => p.to_string_lossy().to_string(),
            Err(_) => entry_path.to_string_lossy().to_string(),
        };
        
        // 分析文件
        let lines: Vec<&str> = content.lines().collect();
        
        // 计算实际代码行（排除空行和注释行）
        let code_lines = count_actual_code_lines(&lines, &extension);
        total_code_lines += code_lines;
        file_line_counts.insert(file_path.clone(), code_lines);
        
        let line_count = lines.len();
        let has_imports = has_import_statements(&content, &extension);
        let has_functions = has_function_definitions(&content, &extension);
        
        // 统计导入语句
        let use_count = match extension.as_str() {
            "rs" => count_use_statements(&content),
            _ => 0,
        };
        total_use_statements += use_count;
        
        // 统计每个依赖的使用情况
        for (name, _) in &dependencies {
            // 识别使用该依赖的行号，并存储为集合以避免重复计数
            let used_lines = identify_dependency_usage_lines(name, &lines, &extension);
            
            if !used_lines.is_empty() {
                let file_map = dependency_usage_map.get_mut(name).unwrap();
                file_map.insert(file_path.clone(), used_lines.into_iter().collect());
            }
        }
        
        // 获取文件大小
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
    
    println!("项目总代码行数（不含空行和注释）: {}", total_code_lines);
    
    // 构建依赖使用情况
    let mut dependency_usage = Vec::new();
    for (name, version) in dependencies {
        // 计算该依赖在所有文件中使用的唯一行数总和
        let mut total_used_lines = 0;
        let mut import_count = 0;
        
        if let Some(files_map) = dependency_usage_map.get(&name) {
            for (file_path, lines) in files_map {
                total_used_lines += lines.len();
                
                // 统计导入语句数量（一般在文件开头）
                if let Some(file_code_lines) = file_line_counts.get(file_path) {
                    if *file_code_lines > 0 {
                        for &line_num in lines.iter() {
                            if line_num <= 20 { // 假设导入语句通常在前20行
                                import_count += 1;
                            }
                        }
                    }
                }
            }
        }
        
        // 计算使用百分比，基于实际代码行数
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
    
    // 按使用行数排序
    dependency_usage.sort_by(|a, b| b.used_lines.cmp(&a.used_lines));
    
    Ok(ProjectAnalysis {
        files: code_files,
        dependency_usage,
        total_use_statements,
        project_type,
    })
}

/// 递归遍历目录
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

/// 查找项目中的 Cargo.lock 文件
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

/// 解析 Cargo.lock 文件，获取依赖包的名称和版本
fn parse_cargo_lock(lock_path: &Path) -> Result<Vec<(String, String)>> {
    let content = fs::read_to_string(lock_path)?;
    let mut packages = Vec::new();
    
    // 使用toml库解析Cargo.lock文件
    let lock_file: toml::Value = content.parse()?;
    
    // 在Cargo.lock中，包信息存储在"package"数组中
    if let Some(package_array) = lock_file.get("package").and_then(|p| p.as_array()) {
        for package in package_array {
            if let (Some(name), Some(version)) = (
                package.get("name").and_then(|n| n.as_str()),
                package.get("version").and_then(|v| v.as_str())
            ) {
                packages.push((name.to_string(), version.to_string()));
            }
        }
    }
    
    Ok(packages)
}

/// 统计指定包在内容中的使用次数
fn count_package_usage(package_name: &str, content: &str) -> usize {
    let re = Regex::new(&format!(r"use\s+{}(?:::|;|\s|$)|\b{}\b::", 
                                 regex::escape(package_name), 
                                 regex::escape(package_name))).unwrap();
    re.find_iter(content).count()
}

/// 统计内容中的 use 语句总数
fn count_use_statements(content: &str) -> usize {
    let re = Regex::new(r"use\s+[^;]+;").unwrap();
    re.find_iter(content).count()
}

/// 检查文件是否为代码文件
fn is_code_file(path: &Path) -> bool {
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        let code_extensions = [
            "rs", "cairo", "js", "ts", "py", "go", "c", "cpp", "h", "hpp", 
            "java", "kt", "swift", "sol", "move", "sh", "bat", "html", "css",
            "json", "toml", "yaml", "yml"
        ];
        
        code_extensions.contains(&ext)
    } else {
        false
    }
}

/// 检查文件是否包含导入语句
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

/// 检查文件是否包含函数定义
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

/// 检测项目类型并获取相应的依赖
fn detect_project_and_dependencies(path: &Path, project_type: &mut String) -> Result<Vec<(String, String)>> {
    // 尝试检测 Rust 项目
    if let Ok(cargo_lock_path) = find_cargo_lock(path) {
        *project_type = "rust".to_string();
        return parse_cargo_lock(&cargo_lock_path);
    }
    
    // 如果没有找到依赖文件，返回空列表而不是错误
    println!("警告: 未找到依赖文件，将使用空依赖列表");
    Ok(Vec::new())
}


/// 计算实际代码行数（排除空行和注释行）
fn count_actual_code_lines(lines: &[&str], extension: &str) -> usize {
    let mut count = 0;
    let mut in_multi_line_comment = false;
    
    for line in lines {
        let trimmed = line.trim();
        
        // 跳过空行
        if trimmed.is_empty() {
            continue;
        }
        
        match extension {
            "py" => {
                // 跳过Python单行注释
                if !trimmed.starts_with("#") {
                    count += 1;
                }
            },
            "rs" => {
                // 处理Rust多行注释
                if in_multi_line_comment {
                    if trimmed.contains("*/") {
                        in_multi_line_comment = false;
                    }
                    continue;
                }
                
                // 处理开始的多行注释
                if trimmed.starts_with("/*") && !trimmed.contains("*/") {
                    in_multi_line_comment = true;
                    continue;
                }
                
                // 跳过单行注释，但计算代码+注释的行
                if !trimmed.starts_with("//") && !(trimmed.starts_with("/*") && trimmed.contains("*/")) {
                    count += 1;
                }
            },
            "js" | "ts" => {
                // 处理JS/TS多行注释
                if in_multi_line_comment {
                    if trimmed.contains("*/") {
                        in_multi_line_comment = false;
                    }
                    continue;
                }
                
                // 处理开始的多行注释
                if trimmed.starts_with("/*") && !trimmed.contains("*/") {
                    in_multi_line_comment = true;
                    continue;
                }
                
                // 跳过单行注释
                if !trimmed.starts_with("//") && !(trimmed.starts_with("/*") && trimmed.contains("*/")) {
                    count += 1;
                }
            },
            _ => {
                // 对于其他扩展名，简单计算非空行
                count += 1;
            }
        }
    }
    
    count
}

/// 识别代码中使用依赖的行号
fn identify_dependency_usage_lines(package_name: &str, lines: &[&str], extension: &str) -> Vec<usize> {
    let mut used_lines = Vec::new();
    
    // 包名称小写化，用于不区分大小写的匹配
    let package_lower = package_name.to_lowercase();
    
    match extension {
        "rs" => {
            // 首先识别导入的函数和类型
            let mut imported_items: Vec<String> = Vec::new();
            
            // 匹配导入语句，如 use demo_dependency_rust::{hello, sum};
            let import_re = Regex::new(&format!(
                r"use\s+{}(?:::|\s*\{{\s*)([^}}]*)(?:\s*}})?", 
                regex::escape(package_name)
            )).unwrap();
            
            for line in lines {
                if let Some(captures) = import_re.captures(line) {
                    if let Some(items_match) = captures.get(1) {
                        let items_str = items_match.as_str();
                        // 分割导入的项，并去除空格
                        for item in items_str.split(',') {
                            let clean_item = item.trim().split(' ').next().unwrap_or("").trim();
                            if !clean_item.is_empty() {
                                imported_items.push(clean_item.to_string());
                            }
                        }
                    }
                }
            }
            
            // 识别四种模式：
            // 1. 直接导入: use package_name::<something>
            // 2. 嵌套导入: use <something>::package_name::<something>
            // 3. 使用路径: package_name::<something>
            // 4. 宏调用: package_name!
            let patterns = [
                // 导入模式
                format!(r"^\s*use\s+{}(?:::|;|\s|$)", regex::escape(package_name)),
                format!(r"^\s*use\s+.*::{}", regex::escape(package_name)),
                // 路径使用模式
                format!(r"\b{}\b::", regex::escape(package_name)),
                // 宏调用模式
                format!(r"\b{}\!", regex::escape(package_name)),
            ];
            
            let patterns: Vec<Regex> = patterns.iter()
                .map(|p| Regex::new(p).unwrap())
                .collect();
            
            // 创建导入项的使用模式
            let imported_item_patterns: Vec<Regex> = imported_items.iter()
                .map(|item| {
                    // 匹配函数调用、结构体实例化或类型声明
                    Regex::new(&format!(r"\b{}\s*(?:\(|\{{|:)", regex::escape(item))).unwrap()
                })
                .collect();
            
            // 特殊处理常见的Rust库
            let special_patterns = match package_lower.as_str() {
                "serde" => vec![
                    Regex::new(r"#\[derive\(.*Serialize.*\)\]").unwrap(),
                    Regex::new(r"#\[derive\(.*Deserialize.*\)\]").unwrap(),
                ],
                "tokio" => vec![
                    Regex::new(r"#\[tokio::main\]").unwrap(),
                    Regex::new(r"tokio::spawn").unwrap(),
                ],
                "anyhow" => vec![
                    Regex::new(r"anyhow!\(").unwrap(),
                ],
                "thiserror" => vec![
                    Regex::new(r"#\[derive\(.*Error.*\)\]").unwrap(),
                ],
                _ => vec![],
            };
            
            for (i, line) in lines.iter().enumerate() {
                let mut matched = false;
                
                // 检查标准模式
                for pattern in &patterns {
                    if pattern.is_match(line) {
                        used_lines.push(i + 1); // 行号从1开始
                        matched = true;
                        break;
                    }
                }
                
                // 如果没有匹配到标准模式，检查导入项使用
                if !matched && !imported_item_patterns.is_empty() {
                    for pattern in &imported_item_patterns {
                        if pattern.is_match(line) {
                            used_lines.push(i + 1);
                            matched = true;
                            break;
                        }
                    }
                }
                
                // 如果有特殊处理且仍未匹配，检查特殊模式
                if !matched && !special_patterns.is_empty() {
                    for pattern in &special_patterns {
                        if pattern.is_match(line) {
                            used_lines.push(i + 1);
                            break;
                        }
                    }
                }
            }
        },
        "py" => {
            // 匹配 import package 或 from package import 语句
            let import_re = Regex::new(&format!(r"^\s*import\s+{}(\s|$|\.|,)|^\s*from\s+{}(\s|$|\.)", 
                                      regex::escape(package_name), 
                                      regex::escape(package_name))).unwrap();
            
            // 匹配使用包的情况 - 更全面的检测
            let usage_re = Regex::new(&format!(r"\b{}\.\w+", regex::escape(package_name))).unwrap();
            
            // 添加对内联包使用的检测 (如 pd.DataFrame())
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
                    used_lines.push(i + 1); // 行号从1开始
                } else if let Some(re) = &inline_re {
                    if re.is_match(line) {
                        used_lines.push(i + 1);
                    }
                }
            }
        },
        "js" | "ts" => {
            // 匹配 import 语句 - 修复正则表达式语法
            let import_pattern = format!(
                r#"import.*from\s+['"]{0}['"]|require\(['"]{0}['"]\)"#,
                regex::escape(package_name)
            );
            let import_re = Regex::new(&import_pattern).unwrap();
            
            // 匹配包使用 - 为常见库提供特殊处理
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
        },
        _ => {}
    }
    
    used_lines
}

/// 统计 JavaScript/TypeScript 文件中的 import 语句
fn count_js_imports(content: &str) -> usize {
    let import_re = Regex::new(r"^\s*(import|require)\b").unwrap();
    content.lines().filter(|line| import_re.is_match(line)).count()
}

/// 将ProjectAnalysis转换为简化的库使用情况列表
pub fn to_library_usage(analysis: &ProjectAnalysis) -> Vec<LibraryUsage> {
    analysis.dependency_usage.iter()
        .map(|dep| LibraryUsage {
            name: dep.name.clone(),
            used_lines: dep.used_lines,
            percentage: dep.percentage,
        })
        .collect()
} 