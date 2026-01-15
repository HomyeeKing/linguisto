use linguist::{detect_language_by_extension, detect_language_by_filename};
use serde::Serialize;
use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::fs;
use std::path::Path;

#[derive(Serialize)]
struct LanguageStat {
    lang: String,
    count: usize,
    ratio: f64,
}

/// 递归遍历目录并收集所有文件
fn collect_files(dir_path: &Path) -> Result<Vec<String>, Box<dyn Error>> {
    let mut files = Vec::new();

    if !dir_path.exists() {
        return Err(format!("目录不存在: {}", dir_path.display()).into());
    }

    if !dir_path.is_dir() {
        return Err(format!("不是一个目录: {}", dir_path.display()).into());
    }

    for entry in fs::read_dir(dir_path)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            let dir_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

            if dir_name.starts_with('.')
                || dir_name == "node_modules"
                || dir_name == "target"
                || dir_name == "dist"
                || dir_name == "build"
            {
                continue;
            }

            files.extend(collect_files(&path)?);
        } else {
            if let Some(file_path) = path.to_str() {
                files.push(file_path.to_string());
            }
        }
    }

    Ok(files)
}

/// 检测单个文件的语言
fn detect_file_language(file_path: &str) -> Option<String> {
    let content = fs::read_to_string(file_path).ok();

    // 1. 尝试通过文件名检测
    if let Ok(languages) = detect_language_by_filename(file_path) {
        if !languages.is_empty() {
            if languages.len() > 1 {
                if let Some(content_str) = &content {
                    if let Ok(disambiguated) = linguist::disambiguate(file_path, content_str) {
                        if let Some(lang) = disambiguated.first() {
                            return Some(lang.name.to_string());
                        }
                    }
                }
            }
            return Some(languages[0].name.to_string());
        }
    }

    // 2. 尝试通过扩展名检测
    let path = Path::new(file_path);
    if path.extension().and_then(|ext| ext.to_str()).is_some() {
        if let Ok(languages) = detect_language_by_extension(file_path) {
            if !languages.is_empty() {
                if languages.len() > 1 {
                    if let Some(content_str) = &content {
                        if let Ok(disambiguated) = linguist::disambiguate(file_path, content_str) {
                            if let Some(lang) = disambiguated.first() {
                                return Some(lang.name.to_string());
                            }
                        }
                    }
                }
                return Some(languages[0].name.to_string());
            }
        }
    }

    None
}

/// 统计语言分布并以 JSON 格式输出
fn analyze_directory(dir_path: &Path) -> Result<(), Box<dyn Error>> {
    let files = collect_files(dir_path)?;
    let mut language_stats: HashMap<String, usize> = HashMap::new();
    let total_files = files.len();

    if total_files == 0 {
        println!("[]");
        return Ok(());
    }

    for file_path in &files {
        if let Some(language) = detect_file_language(file_path) {
            *language_stats.entry(language).or_insert(0) += 1;
        }
    }

    let mut result: Vec<LanguageStat> = language_stats
        .into_iter()
        .map(|(lang, count)| LanguageStat {
            lang,
            count,
            ratio: (count as f64 / total_files as f64),
        })
        .collect();

    // 按文件数量降序排列
    result.sort_by(|a, b| b.count.cmp(&a.count));

    println!("{}", serde_json::to_string_pretty(&result)?);

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();

    let dir_path = if args.len() > 1 {
        Path::new(&args[1])
    } else {
        Path::new(".")
    };

    analyze_directory(dir_path)?;

    Ok(())
}
