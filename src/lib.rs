#![deny(clippy::all)]

use ignore::WalkBuilder;
use linguist::{detect_language_by_extension, detect_language_by_filename, is_vendored};
use napi::bindgen_prelude::*;
use napi_derive::napi;
use rayon::prelude::*;
use serde::Serialize;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;

#[napi(object)]
#[derive(Serialize)]
pub struct LanguageStat {
  pub lang: String,
  pub count: u32,
  pub bytes: i64,
  pub ratio: f64,
}

const READ_LIMIT: usize = 32768; // 32KB

/// 读取文件头部内容用于检测
fn read_file_header(file_path: &str) -> Option<String> {
  let mut file = File::open(file_path).ok()?;
  let mut buffer = vec![0; READ_LIMIT];
  let n = file.read(&mut buffer).ok()?;
  String::from_utf8(buffer[..n].to_vec()).ok()
}

/// 并行收集所有文件及其大小
fn collect_files_parallel(dir_path: &Path) -> Vec<(String, i64)> {
  let (tx, rx) = std::sync::mpsc::channel();

  let walker = WalkBuilder::new(dir_path)
    .hidden(true)
    .git_ignore(true)
    .threads(num_cpus::get()) // 使用多线程遍历
    .build_parallel();

  walker.run(|| {
    let tx = tx.clone();
    Box::new(move |result| {
      if let Ok(entry) = result {
        if entry.file_type().map_or(false, |ft| ft.is_file()) {
          let path = entry.path();
          if !is_vendored(path).unwrap_or(false) {
            if let Ok(metadata) = entry.metadata() {
              if let Some(path_str) = path.to_str() {
                let _ = tx.send((path_str.to_string(), metadata.len() as i64));
              }
            }
          }
        }
      }
      ignore::WalkState::Continue
    })
  });

  drop(tx);
  rx.into_iter().collect()
}

/// 我们最终统计中是否要计入该语言
/// - Programming: 全部计入
/// - Markup: 只计入 HTML / CSS
/// - 其它类型（Data / Prose / Text 等）不计入
fn should_include_language(name: &str, lang_type: &linguist_types::LanguageType) -> bool {
  match lang_type {
    linguist_types::LanguageType::Programming => true,
    linguist_types::LanguageType::Markup => {
      let lower = name.to_lowercase();
      lower == "html" || lower == "css"
    }
    _ => false,
  }
}

/// 检测单个文件的语言（优化版：按需读取头部）
/// 返回：规范化后的语言名，以及其 LanguageType（仅内部使用）
fn detect_file_language(file_path: &str) -> Option<(String, linguist_types::LanguageType)> {
  // 1. 尝试通过文件名检测
  if let Ok(languages) = detect_language_by_filename(file_path) {
    if !languages.is_empty() {
      let final_lang = if languages.len() > 1 {
        if let Some(content_str) = read_file_header(file_path) {
          linguist::disambiguate(file_path, &content_str)
            .ok()
            .and_then(|d| d.first().cloned())
            .unwrap_or_else(|| languages[0].clone())
        } else {
          languages[0].clone()
        }
      } else {
        languages[0].clone()
      };

      let mut name = final_lang.name.to_string();
      // 将 TSX 归类为 TypeScript，避免在结果中单独显示
      if name.to_lowercase() == "tsx" {
        name = "TypeScript".to_string();
      }
      return Some((name, final_lang.definition.language_type.clone()));
    }
  }

  // 2. 尝试通过扩展名检测
  let path = Path::new(file_path);
  if path.extension().and_then(|ext| ext.to_str()).is_some() {
    if let Ok(languages) = detect_language_by_extension(file_path) {
      if !languages.is_empty() {
        let final_lang = if languages.len() > 1 {
          if let Some(content_str) = read_file_header(file_path) {
            linguist::disambiguate(file_path, &content_str)
              .ok()
              .and_then(|d| d.first().cloned())
              .unwrap_or_else(|| languages[0].clone())
          } else {
            languages[0].clone()
          }
        } else {
          languages[0].clone()
        };

        let mut name = final_lang.name.to_string();
        // 将 TSX 归类为 TypeScript，避免在结果中单独显示
        if name.to_lowercase() == "tsx" {
          name = "TypeScript".to_string();
        }
        return Some((name, final_lang.definition.language_type.clone()));
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
  let files = collect_files_parallel(path);
  let total_bytes: i64 = files.iter().map(|f| f.1).sum();

  if total_bytes == 0 {
    return Vec::new();
  }
  
  // 并行检测语言
  let language_stats = files
    .par_iter()
    .fold(
      HashMap::new,
      |mut acc: HashMap<String, (u32, i64)>, (file_path, size)| {
        if let Some((language, lang_type)) = detect_file_language(file_path) {
          if !should_include_language(&language, &lang_type) {
            return acc;
          }

          let entry = acc.entry(language).or_insert((0u32, 0i64));
          entry.0 += 1;
          entry.1 += *size;
        }
        acc
      },
    )
    .reduce(HashMap::new, |mut acc1, acc2| {
      for (lang, (count, bytes)) in acc2 {
        let entry = acc1.entry(lang).or_insert((0, 0));
        entry.0 += count;
        entry.1 += bytes;
      }
      acc1
    });

  let mut result: Vec<LanguageStat> = language_stats
    .into_iter()
    .map(|(lang, (count, bytes))| LanguageStat {
      lang,
      count,
      bytes,
      ratio: (bytes as f64 / total_bytes as f64),
    })
    .collect();

  // 按字节数降序排列
  result.sort_by(|a, b| b.bytes.cmp(&a.bytes));

  result
}
