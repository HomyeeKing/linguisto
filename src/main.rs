use colored::*;
use linguisto::analyze_directory;
use std::env;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();

    let mut json_mode = false;
    let mut dir_path = ".".to_string();

    // 简单的参数解析
    for arg in args.iter().skip(1) {
        if arg == "--json" {
            json_mode = true;
        } else if !arg.starts_with("--") {
            dir_path = arg.clone();
        }
    }

    let result = analyze_directory(dir_path);

    if json_mode {
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        render_terminal_ui(&result);
    }

    Ok(())
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
        _ => Color::TrueColor {
            r: 133,
            g: 133,
            b: 133,
        },
    }
}

fn render_terminal_ui(stats: &[linguisto::LanguageStat]) {
    if stats.is_empty() {
        println!("{}", "未发现识别的代码文件".bright_black());
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
