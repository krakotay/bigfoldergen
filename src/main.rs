use byte_unit::{Byte, ParseError};
use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};
use rand::distributions::Alphanumeric;
use rand::Rng;
use std::error::Error;
use std::fmt;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};

type BigError = Box<dyn Error>;

#[derive(Debug)]
enum MyError {
    SizeParseError(String),
}

impl fmt::Display for MyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MyError::SizeParseError(msg) => write!(f, "{}", msg),
        }
    }
}

impl Error for MyError {}

impl From<ParseError> for MyError {
    fn from(error: ParseError) -> Self {
        match error {
            ParseError::Unit(unit_error) => {
                let expected_chars: String = unit_error.expected_characters.iter().collect();
                MyError::SizeParseError(format!(
                    "Неверная единица измерения '{}'. Ожидались: {}",
                    unit_error.character, expected_chars
                ))
            }
            _ => MyError::SizeParseError(error.to_string()),
        }
    }
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// destination folder (with -f/--folder-path)
    #[arg(short = 'f', long, value_name = "FOLDER")]
    folder_path: Option<String>,

    /// destination folder (positional argument)
    #[arg(index = 1)]
    positional_folder_path: Option<String>,

    /// Size of folder (with -s/--size)
    #[arg(short = 's', long, value_name = "SIZE")]
    size: Option<String>,

    /// Size of folder (positional argument)
    #[arg(index = 2)]
    positional_size: Option<String>,

    /// Depth of recurse
    #[arg(short, long, default_value_t = 1)]
    d: u64,

    /// minimum size of file
    #[arg(short = 'm', long, default_value_t = String::from("1kib"))]
    min: String,
    /// maximum size of file
    #[arg(short = 'M', long, default_value_t = String::from("5kib"))]
    max: String,
}

fn main() -> Result<(), BigError> {
    let args = Args::parse();
    let folder_path = args
        .folder_path
        .or(args.positional_folder_path)
        .ok_or_else(|| BigError::from("Folder path must be specified"))?;
    let folder_path = Path::new(&folder_path).to_path_buf();
    let size = args
        .size
        .or(args.positional_size)
        .ok_or_else(|| BigError::from("Size be specified"))?;
    let total_size = parse_size(&size)?.as_u64();
    let depth = args.d;
    let min_file_size = parse_size(&args.min)?.as_u64();
    let mut max_file_size = parse_size(&args.max)?.as_u64();
    if max_file_size < min_file_size {
        eprintln!("min is more than max! I'l make them equal.");
        max_file_size = min_file_size;
    }
    let mut rng = rand::thread_rng();
    let mut current_size = 0u64;
    let mut file_number = 0u64;

    let pb = ProgressBar::new(total_size);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
            .unwrap()
            .progress_chars("#>-"),
    );


    fs::create_dir_all(&folder_path)?;
    while current_size < total_size {
        let file_size = rng.gen_range(min_file_size..=max_file_size);
        let file_name = format!("file_{}.txt", file_number);
        let nested_depth = rng.gen_range(0..=depth);
        let file_path = create_nested_path(&folder_path, nested_depth, &mut rng);

        fs::create_dir_all(&file_path)?;
        let full_file_path = file_path.join(file_name);

        let mut file = File::create(full_file_path)?;
        let content: Vec<u8> = (0..file_size).map(|_| rng.gen::<u8>()).collect();
        file.write_all(&content)?;

        current_size += file_size;
        file_number += 1;
        pb.set_position(current_size);
    }

    pb.finish_with_message("Done");

    println!("Created files: {}", file_number);
    println!(
        "Total size: {:.2} GiB",
        current_size as f64 / 1024.0 / 1024.0 / 1024.0
    );

    Ok(())
}

fn create_nested_path(base_path: &Path, depth: u64, rng: &mut impl Rng) -> PathBuf {
    let mut current_path = base_path.to_path_buf();
    for _ in 0..depth {
        let folder_name: String = rng
            .sample_iter(&Alphanumeric)
            .take(8)
            .map(char::from)
            .collect();
        current_path = current_path.join(folder_name);
    }
    current_path
}

fn parse_size(s: &str) -> Result<Byte, MyError> {
    Byte::parse_str(s, true).map_err(MyError::from)
}
