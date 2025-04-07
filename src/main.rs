// This is free and unencumbered software released into the public domain.

#![deny(unsafe_code)]

mod commands;

use clientele::{
    crates::clap::{Parser, Subcommand},
    StandardOptions,
    SysexitsError::{self, *},
};

/// ASIMOV Module Command-Line Interface (CLI)
#[derive(Debug, Parser)]
#[command(name = "asimov-module", long_about)]
#[command(arg_required_else_help = true)]
struct Options {
    #[clap(flatten)]
    flags: StandardOptions,

    #[clap(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// TBD
    #[clap(alias = "ls")]
    List {},
}

pub fn main() -> SysexitsError {
    // Load environment variables from `.env`:
    clientele::dotenv().ok();

    // Expand wildcards and @argfiles:
    let Ok(args) = clientele::args_os() else {
        return EX_USAGE;
    };

    // Parse command-line options:
    let options = Options::parse_from(&args);

    // Print the version, if requested:
    if options.flags.version {
        println!("asimov-module {}", env!("CARGO_PKG_VERSION"));
        return EX_OK;
    }

    // Print the license, if requested:
    if options.flags.license {
        print!("{}", include_str!("../UNLICENSE"));
        return EX_OK;
    }

    // Configure debug output:
    if options.flags.debug {
        std::env::set_var("RUST_BACKTRACE", "1");
    }

    // Execute the given command:
    let result = match options.command.unwrap() {
        Command::List {} => commands::list(&options.flags),
    };

    match result {
        Ok(()) => EX_OK,
        Err(err) => err,
    }
}
