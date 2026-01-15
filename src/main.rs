use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use linguisto::{analyze_directory, LanguageStat};
use std::env;
use std::error::Error;
use std::time::Duration;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();

    let mut json_mode = false;
    let mut show_all = false;
    let mut sort_by_bytes = true;
    let mut dir_path = ".".to_string();
    let mut max_lang = 6;

    // 简单的参数解析
    let mut i = 1;
    while i < args.len() {
        let arg = &args[i];
        if arg == "--json" {
            json_mode = true;
        } else if arg == "--all" {
            show_all = true;
        } else if arg == "--sort=file_count" {
            sort_by_bytes = false;
        } else if arg == "--sort=bytes" {
            sort_by_bytes = true;
        } else if arg.starts_with("--max-lang=") {
            if let Ok(val) = arg.replace("--max-lang=", "").parse::<usize>() {
                max_lang = val;
            }
        } else if arg == "--max-lang" && i + 1 < args.len() {
            if let Ok(val) = args[i + 1].parse::<usize>() {
                max_lang = val;
                i += 1;
            }
        } else if !arg.starts_with("--") {
            dir_path = arg.clone();
        }
        i += 1;
    }

    // 显示进度条（使用 stderr，不会影响 JSON 输出）
    let pb = ProgressBar::new_spinner();
    pb.enable_steady_tick(Duration::from_millis(120));
    pb.set_style(
        ProgressStyle::with_template("{spinner:.blue} {msg}")
            .unwrap()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]),
    );
    pb.set_message("Analyzing directory...");

    let raw_result = analyze_directory(dir_path);

    pb.finish_and_clear();
    
    // 无论是默认还是 --all，都根据排序策略重新计算比例
    let stats = if show_all {
        recalculate_ratios(raw_result, sort_by_bytes)
    } else {
        process_stats_for_ui(raw_result, sort_by_bytes)
    };

    let final_result = collapse_tail_to_others(stats, max_lang);

    if json_mode {
        println!("{}", serde_json::to_string_pretty(&final_result)?);
    } else {
        render_terminal_ui(&final_result);
    }

    Ok(())
}

fn collapse_tail_to_others(mut stats: Vec<LanguageStat>, max_lang: usize) -> Vec<LanguageStat> {
    if stats.len() <= max_lang || max_lang == 0 {
        return stats;
    }

    // 保留前 max_lang 个，剩下的合并为 Other
    let mut result: Vec<LanguageStat> = stats.drain(0..max_lang).collect();

    let mut others_count = 0;
    let mut others_bytes = 0;
    let mut others_ratio = 0.0;

    for stat in stats {
        others_count += stat.count;
        others_bytes += stat.bytes;
        others_ratio += stat.ratio;
    }
  
  result.push(LanguageStat {
    lang: "Other".to_string(),
    count: others_count,
    bytes: others_bytes,
    ratio: others_ratio,
  });

    result
}

fn recalculate_ratios(mut stats: Vec<LanguageStat>, sort_by_bytes: bool) -> Vec<LanguageStat> {
    if sort_by_bytes {
        let total_bytes: i64 = stats.iter().map(|s| s.bytes).sum();
        if total_bytes > 0 {
            for stat in &mut stats {
                stat.ratio = stat.bytes as f64 / total_bytes as f64;
            }
        }
        stats.sort_by(|a, b| b.bytes.cmp(&a.bytes));
    } else {
        let total_count: u32 = stats.iter().map(|s| s.count).sum();
        if total_count > 0 {
            for stat in &mut stats {
                stat.ratio = stat.count as f64 / total_count as f64;
            }
        }
        stats.sort_by(|a, b| b.count.cmp(&a.count));
    }
    stats
}

fn process_stats_for_ui(stats: Vec<LanguageStat>, sort_by_bytes: bool) -> Vec<LanguageStat> {
    // Rust 侧已经对语言类型做了筛选，这里只负责重新计算比例并排序
    recalculate_ratios(stats, sort_by_bytes)
}

fn get_color(lang: &str) -> Color {
    match lang.to_lowercase().as_str() {
        "rust" => Color::TrueColor {
            r: 222,
            g: 165,
            b: 132,
        },
        "javascript" => Color::TrueColor {
            r: 241,
            g: 224,
            b: 90,
        },
        "typescript" => Color::TrueColor {
            r: 49,
            g: 108,
            b: 194,
        },
        "python" => Color::TrueColor {
            r: 53,
            g: 114,
            b: 165,
        },
        "html" => Color::TrueColor {
            r: 227,
            g: 76,
            b: 38,
        },
        "css" => Color::TrueColor {
            r: 86,
            g: 61,
            b: 124,
        },
        "go" => Color::TrueColor {
            r: 0,
            g: 173,
            b: 216,
        },
        "json" | "json with comments" => Color::TrueColor {
            r: 41,
            g: 134,
            b: 102,
        },
        "markdown" => Color::TrueColor {
            r: 8,
            g: 63,
            b: 161,
        },
        "other" => Color::TrueColor {
            r: 133,
            g: 133,
            b: 133,
        },
        _ => Color::TrueColor {
            r: 133,
            g: 133,
            b: 133,
        },
    }
}

fn render_terminal_ui(stats: &[LanguageStat]) {
    if stats.is_empty() {
        println!("{}", "未发现识别的文件".bright_black());
        return;
    }

    let bar_width = 60;
    let mut bar_str = String::new();
    let mut current_width = 0;

    // 1. 绘制进度条
    for (i, stat) in stats.iter().enumerate() {
        let color = get_color(&stat.lang);
        let width = if i == stats.len() - 1 {
            bar_width - current_width
        } else {
            (stat.ratio * bar_width as f64).round() as usize
        };

        if width > 0 {
            let segment = "█".repeat(width);
            bar_str.push_str(&segment.color(color).to_string());
            current_width += width;
        }
    }

    println!("\n{}\n", bar_str);

    // 2. 绘制图例
    let mut legend_lines = Vec::new();
    let mut current_line = String::new();

    for stat in stats {
        let color = get_color(&stat.lang);
        let bullet = "●".color(color);
        let text = format!("{} {} {:.1}%   ", bullet, stat.lang.bold(), stat.ratio * 100.0);

        if current_line.len() + text.len() > 80 {
            legend_lines.push(current_line);
            current_line = text;
        } else {
            current_line.push_str(&text);
        }
    }
    if !current_line.is_empty() {
        legend_lines.push(current_line);
    }

    for line in legend_lines {
        println!("{}", line);
    }
    println!();
}
