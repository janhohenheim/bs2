#![expect(unused_variables, reason = "WIP")]
use clap::{Parser, Subcommand};

/// A fictional versioning CLI
#[derive(Debug, Parser)] // requires `derive` feature
#[command(name = "bs2")]
#[command(about = "Bevy <-> Source 2 Tools Integration", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Sets up the Source 2 Tools by copying files from an existing installation of Counter Strike 2. Needs to be run at least once in order for bs2 to work.
    Setup {
        /// The path to Counter Strike 2.
        #[arg(long, value_name = "PATH")]
        #[cfg_attr(
            target_os = "windows",
            arg(
                default_value = "C:/Program Files (x86)/Steam/steamapps/common/Counter-Strike Global Offensive"
            )
        )]
        _in: String,
        /// The path where the Source 2 Tools should be extracted
        #[arg(
            long,
            value_name = "PATH",
            default_value = r"~/AppData/Local/Source 2 Tools/"
        )]
        out: String,
        /// Performs the setup even if the output directory already exists. Useful when performing an upgrade or fixing a broken installation.
        #[arg(short('f'), long)]
        force: bool,
    },
    /// Initializes a new addon
    Init {
        /// The addon to add. Will use the project name by default.
        name: Option<String>,
    },
    /// Launches the Source 2 Tools
    Tools,
    /// Renames an addon
    Rename {
        /// The addon to rename. Will use the project name by default.
        old: Option<String>,
        /// The new name of the addon. Addon names must confirm to the same rules as crate names.
        #[arg(required = true)]
        name: String,
    },
    /// Removes an addon
    Remove {
        /// The addon to remove. Will use the project name by default.
        name: Option<String>,
    },
    /// Removes all Source 2 Tools related files, including all addons.
    #[command(arg_required_else_help = true)]
    Purge {
        /// Confirm that you know you're about to delete all your addons. WARNING: THIS WILL REMOVE ALL YOUR MAPS, ETC.
        #[arg(required = true, long)]
        confirm: bool,
    },
}

fn main() {
    let args = Cli::parse();

    match args.command {
        Commands::Setup { _in, out, force } => todo!(),
        Commands::Init { name } => todo!(),
        Commands::Tools => todo!(),
        Commands::Rename { old, name } => todo!(),
        Commands::Remove { name } => todo!(),
        Commands::Purge { confirm } => todo!(),
    }

    // Continued program logic goes here...
}
