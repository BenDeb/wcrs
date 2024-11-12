use clap::Parser;
use std::process;

use wc::Cli;
use wc::Config;

fn main() {
    let args = Cli::parse();

    // Build the configuration from args.
    let conf = Config::build(args).unwrap_or_else(|err| {
        println!("There was an issue: {err}");
        process::exit(1);
    });
    if let Err(e) = wc::run(conf) {
        eprintln!("There was an issue during execution: {e}");
        process::exit(1);
    }
}
