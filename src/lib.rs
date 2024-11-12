use clap::Parser;
use std::error::Error;
use std::fmt::Write as _;
use std::fs::File;
use std::io::{self, BufRead, BufReader, IsTerminal};
use std::process::exit;
use std::str::{self};

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    /// The file path to analyze.
    files: Vec<String>,

    /// Counting bytes.
    #[arg(short = 'c', long)]
    bytes: bool,

    /// Counting lines.
    #[arg(short = 'l', long)]
    lines: bool,

    /// Counting words.
    #[arg(short = 'w', long)]
    words: bool,

    /// Counting chars.
    #[arg(short = 'm', long)]
    chars: bool,
}

#[derive(Debug)]
pub struct Config {
    files_paths: Vec<String>,
    lines: bool,
    words: bool,
    bytes: bool,
    chars: bool,
    all: bool,
}

fn is_all_enabled(args: &Cli) -> bool {
    if args.lines || args.bytes || args.words || args.chars {
        return false;
    }
    true
}

impl Config {
    pub fn build(args: Cli) -> Result<Config, &'static str> {
        let all = is_all_enabled(&args);
        Ok(Config {
            files_paths: args.files,
            lines: args.lines,
            bytes: args.bytes,
            words: args.words,
            chars: args.chars,
            all,
        })
    }
}

fn open(filename: &str) -> MyResult<Box<dyn BufRead>> {
    match filename {
        // If file name is "-", it means we're working on stdin.
        // First we create a lock handle on stdin, to avoid concurrency issues.
        // Then we create a reader over this handle.
        "-" => {
            let stdin_handle = io::stdin().lock();
            if stdin_handle.is_terminal() {
                eprintln!("Must not be a terminal");
                exit(1);
            }
            Ok(Box::new(BufReader::new(stdin_handle)))
        }
        _ => Ok(Box::new(BufReader::with_capacity(
            2048,
            File::open(filename)?,
        ))),
    }
}

// type FileInfos = Vec<FileInfo>;
pub fn run(conf: Config) -> Result<(), std::io::Error> {
    // Total X for ALL files
    let mut global_total_bytes: usize = 0;
    let mut global_total_words: usize = 0;
    let mut global_total_lines: usize = 0;

    for file in &conf.files_paths {
        let input = open(file);
        let results = count(&conf, input.unwrap())?;
        global_total_bytes += results.total_bytes;
        global_total_words += results.total_words;
        global_total_lines += results.total_lines;

        formatter(results, &conf, file).unwrap();
    }
    if conf.files_paths.len() > 1 {
        total_formatter(
            &conf,
            global_total_lines,
            global_total_words,
            global_total_bytes,
        )
        .unwrap();
    }

    Ok(())
}
#[derive(Debug)]
struct FileInfo {
    total_bytes: usize,
    total_chars: usize,
    total_words: usize,
    total_lines: usize,
}
fn formatter(results: FileInfo, conf: &Config, filename: &str) -> Result<(), std::fmt::Error> {
    let mut to_print = String::new();
    let width = 6;
    if conf.all {
        write!(
            to_print,
            "{:width$} {:width$} {:width$}",
            results.total_lines, results.total_words, results.total_bytes
        )?;
    }
    if conf.lines {
        write!(to_print, "{:width$} ", results.total_lines)?;
    }
    if conf.words {
        write!(to_print, "{:width$} ", results.total_words)?;
    }
    if conf.bytes {
        write!(to_print, "{:width$} ", results.total_bytes)?;
    }
    if conf.chars {
        write!(to_print, "{:width$} ", results.total_chars)?;
    }

    print!("{to_print} ");
    println!("{filename}");
    Ok(())
}

fn total_formatter(
    conf: &Config,
    total_lines: usize,
    total_words: usize,
    total_bytes: usize,
) -> Result<(), std::fmt::Error> {
    let mut to_print = String::new();
    let width = 6;

    if conf.all {
        println!(
            "{:width$} {:width$} {:width$} total",
            total_lines, total_words, total_bytes
        );
        return Ok(());
    } else {
        if conf.lines {
            write!(to_print, "{:width$} ", total_lines)?;
        }
        if conf.words {
            write!(to_print, "{:width$} ", total_words)?;
        }
        if conf.bytes {
            write!(to_print, "{:width$} ", total_bytes)?;
        }
    }

    println!("{} total", to_print);
    Ok(())
}

fn count(conf: &Config, mut reader: impl BufRead) -> Result<FileInfo, std::io::Error> {
    let mut total_bytes = 0;
    let mut total_lines = 0;
    let mut total_words = 0;
    let mut total_chars = 0;
    let mut nb_bytes;
    let mut buffer = String::new();
    nb_bytes = reader.read_line(&mut buffer)?;
    total_bytes += nb_bytes;
    while nb_bytes > 0 {
        nb_bytes = reader.read_line(&mut buffer)?;
        total_bytes += nb_bytes;
        total_lines += 1;
        if conf.chars || conf.all {
            total_chars += buffer.chars().count();
        }
        if conf.words || conf.all {
            total_words += buffer.split_whitespace().count();
        }

        buffer.clear();
    }
    Ok(FileInfo {
        total_bytes,
        total_lines,
        total_words,
        total_chars,
    })
}
