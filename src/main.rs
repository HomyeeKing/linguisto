use linguisto::analyze_directory;
use std::env;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();

    let dir_path = if args.len() > 1 {
        &args[1]
    } else {
        "."
    };

    let result = analyze_directory(dir_path.to_string());
    println!("{}", serde_json::to_string_pretty(&result)?);

    Ok(())
}
