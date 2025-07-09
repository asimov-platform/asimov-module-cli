// This is free and unencumbered software released into the public domain.

#![deny(unsafe_code)]

use asimov_module_cli::commands;

use clientele::{
    StandardOptions,
    SysexitsError::{self, *},
    crates::clap::{Parser, Subcommand},
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
    /// Open the module's package page in a web browser
    #[clap(alias = "open")]
    Browse {
        /// The name of the module to browse
        name: String,
    },

    /// TBD
    Config {
        /// The name of the module to configure
        name: String,

        #[clap(trailing_var_arg = true)]
        args: Vec<String>,
    },

    /// TBD
    #[cfg(feature = "unstable")]
    Disable {
        /// The names of the modules to disable
        names: Vec<String>,
    },

    /// TBD
    #[cfg(feature = "unstable")]
    Enable {
        /// The names of the modules to enable
        names: Vec<String>,
    },

    /// TBD
    #[cfg(feature = "unstable")]
    #[clap(alias = "which")]
    Find {
        /// The name of the module to find
        name: String,
    },

    /// TBD
    #[cfg(feature = "unstable")]
    #[clap(alias = "show")]
    Inspect {
        /// The name of the module to inspect
        name: String,
    },

    /// Install an available module locally
    Install {
        /// The names of the modules to install
        names: Vec<String>,
    },

    /// Print the module's package link
    #[clap(alias = "url")]
    Link {
        /// The name of the module to link to
        name: String,
    },

    /// List all available and/or installed modules
    #[clap(alias = "ls")]
    List {},

    /// Resolve a given URL to modules which can handle it
    Resolve {
        /// The URL to resolve
        url: String,
    },

    /// Uninstall a currently installed module
    Uninstall {
        /// The names of the modules to uninstall
        names: Vec<String>,
    },

    /// TBD
    #[cfg(feature = "unstable")]
    #[clap(alias = "update")]
    Upgrade {
        /// The names of the modules to upgrade
        names: Vec<String>,
    },
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

    asimov_module::init_tracing_subscriber(&options.flags).expect("failed to initialize logging");

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

    // Execute the given command:
    let result = match options.command.unwrap() {
        Command::Browse { name } => commands::browse(name, &options.flags),
        Command::Config { name, args } => commands::config(name, &args, &options.flags),
        #[cfg(feature = "unstable")]
        Command::Disable { names } => commands::disable(names, &options.flags),
        #[cfg(feature = "unstable")]
        Command::Enable { names } => commands::enable(names, &options.flags),
        #[cfg(feature = "unstable")]
        Command::Find { name } => commands::find(name, &options.flags),
        #[cfg(feature = "unstable")]
        Command::Inspect { name } => commands::inspect(name, &options.flags),
        Command::Install { names } => commands::install(names, &options.flags),
        Command::Link { name } => commands::link(name, &options.flags),
        Command::List {} => commands::list(&options.flags),
        Command::Resolve { url } => commands::resolve(url, &options.flags),
        Command::Uninstall { names } => commands::uninstall(names, &options.flags),
        #[cfg(feature = "unstable")]
        Command::Upgrade { names } => commands::upgrade(names, &options.flags),
    };

    match result {
        Ok(()) => EX_OK,
        Err(err) => err,
    }
}
