#![deny(clippy::all)]

use ignore::WalkBuilder;
use linguist::{detect_language_by_extension, detect_language_by_filename, is_vendored};
use napi::bindgen_prelude::*;
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
  pub bytes: i64,
  pub ratio: f64,
  pub is_programming: bool,
}

/// 收集所有文件及其大小，自动尊重 .gitignore 并跳过隐藏文件和第三方库
fn collect_files(dir_path: &Path) -> Vec<(String, i64)> {
  let mut files = Vec::new();

  let walker = WalkBuilder::new(dir_path)
    .hidden(true) // 跳过隐藏文件/目录（如 .git）
    .git_ignore(true) // 尊重 .gitignore
    .build();

  for result in walker {
    if let Ok(entry) = result {
      // 只处理文件
      if entry.file_type().map_or(false, |ft| ft.is_file()) {
        let path = entry.path();
        
        // 使用 linguist 原生的 is_vendored 进一步过滤掉第三方依赖（如 vendor/ 目录）
        if is_vendored(path).unwrap_or(false) {
          continue;
        }

        if let Ok(metadata) = entry.metadata() {
          if let Some(path_str) = path.to_str() {
            files.push((path_str.to_string(), metadata.len() as i64));
          }
        }
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

#[napi(js_name = "analyzeDirectorySync")]
pub fn analyze_directory(dir_path: String) -> Vec<LanguageStat> {
  analyze_directory_internal(dir_path)
}

#[napi(js_name = "analyzeDirectory", ts_return_type = "Promise<Array<LanguageStat>>")]
pub fn analyze_directory_async(dir_path: String) -> AsyncTask<AnalyzeTask> {
  AsyncTask::new(AnalyzeTask { dir_path })
}

pub struct AnalyzeTask {
  pub dir_path: String,
}

#[napi]
impl Task for AnalyzeTask {
  type Output = Vec<LanguageStat>;
  type JsValue = Vec<LanguageStat>;

  fn compute(&mut self) -> Result<Self::Output> {
    Ok(analyze_directory_internal(self.dir_path.clone()))
  }

  fn resolve(&mut self, _env: Env, output: Self::Output) -> Result<Self::JsValue> {
    Ok(output)
  }
}

fn analyze_directory_internal(dir_path: String) -> Vec<LanguageStat> {
  let path = Path::new(&dir_path);
  let files = collect_files(path);
  let mut language_stats: HashMap<String, (u32, i64, bool)> = HashMap::new();
  let total_bytes: i64 = files.iter().map(|f| f.1).sum();

  if total_bytes == 0 {
    return Vec::new();
  }

  for (file_path, size) in &files {
    if let Some((language, is_prog)) = detect_file_language(file_path) {
      let entry = language_stats.entry(language).or_insert((0, 0, is_prog));
      entry.0 += 1;
      entry.1 += size;
    }
  }

  let mut result: Vec<LanguageStat> = language_stats
    .into_iter()
    .map(|(lang, (count, bytes, is_programming))| LanguageStat {
      lang,
      count,
      bytes,
      ratio: (bytes as f64 / total_bytes as f64),
      is_programming,
    })
    .collect();

  // 按字节数降序排列
  result.sort_by(|a, b| b.bytes.cmp(&a.bytes));

  result
}
