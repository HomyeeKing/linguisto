#![deny(clippy::all)]

use linguist::{detect_language_by_extension, detect_language_by_filename};
use napi_derive::napi;
use serde::Serialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[napi(object)]
#[derive(Serialize)]
pub struct LanguageStat {
  pub lang: String,
  pub count: u32,
  pub ratio: f64,
  pub is_programming: bool,
}

/// 递归遍历目录并收集所有文件
fn collect_files(dir_path: &Path) -> Vec<String> {
  let mut files = Vec::new();

  if !dir_path.exists() || !dir_path.is_dir() {
    return files;
  }

  if let Ok(entries) = fs::read_dir(dir_path) {
    for entry in entries.flatten() {
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

        files.extend(collect_files(&path));
      } else if let Some(file_path) = path.to_str() {
        files.push(file_path.to_string());
      }
    }
  }

  files
}

/// 检测单个文件的语言
fn detect_file_language(file_path: &str) -> Option<(String, bool)> {
  let content = fs::read_to_string(file_path).ok();

  // 1. 尝试通过文件名检测
  if let Ok(languages) = detect_language_by_filename(file_path) {
    if !languages.is_empty() {
      let lang = if languages.len() > 1 {
        if let Some(content_str) = &content {
          if let Ok(disambiguated) = linguist::disambiguate(file_path, content_str) {
            disambiguated.first().cloned()
          } else {
            None
          }
        } else {
          None
        }
      } else {
        None
      };
      
      let final_lang = lang.unwrap_or_else(|| languages[0].clone());
      return Some((
        final_lang.name.to_string(),
        matches!(final_lang.definition.language_type, linguist_types::LanguageType::Programming)
      ));
    }
  }

  // 2. 尝试通过扩展名检测
  let path = Path::new(file_path);
  if path.extension().and_then(|ext| ext.to_str()).is_some() {
    if let Ok(languages) = detect_language_by_extension(file_path) {
      if !languages.is_empty() {
        let lang = if languages.len() > 1 {
          if let Some(content_str) = &content {
            if let Ok(disambiguated) = linguist::disambiguate(file_path, content_str) {
              disambiguated.first().cloned()
            } else {
              None
            }
          } else {
            None
          }
        } else {
          None
        };

        let final_lang = lang.unwrap_or_else(|| languages[0].clone());
        return Some((
          final_lang.name.to_string(),
          matches!(final_lang.definition.language_type, linguist_types::LanguageType::Programming)
        ));
      }
    }
  }

  None
}

#[napi]
pub fn analyze_directory(dir_path: String) -> Vec<LanguageStat> {
  let path = Path::new(&dir_path);
  let files = collect_files(path);
  let mut language_stats: HashMap<String, (u32, bool)> = HashMap::new();
  let total_files = files.len();

  if total_files == 0 {
    return Vec::new();
  }

  for file_path in &files {
    if let Some((language, is_prog)) = detect_file_language(file_path) {
      let entry = language_stats.entry(language).or_insert((0, is_prog));
      entry.0 += 1;
    }
  }

  let mut result: Vec<LanguageStat> = language_stats
    .into_iter()
    .map(|(lang, (count, is_programming))| LanguageStat {
      lang,
      count,
      ratio: (count as f64 / total_files as f64),
      is_programming,
    })
    .collect();

  // 按文件数量降序排列
  result.sort_by(|a, b| b.count.cmp(&a.count));

  result
}
