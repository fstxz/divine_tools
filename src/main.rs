use std::path::PathBuf;

use argh::FromArgs;

use crate::editor::run_editor;

mod buffer;
mod editor;
mod types;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// Tool for inspecting and modifying various binary files from Divine Divinity and Beyond Divinity.
#[derive(FromArgs)]
struct Args {
    #[argh(subcommand)]
    command: Option<SubCommand>,
}

#[derive(FromArgs)]
#[argh(subcommand)]
enum SubCommand {
    Pack(PackCommand),
    Unpack(UnpackCommand),
}

/// unpacks a .cmp file
#[derive(FromArgs)]
#[argh(subcommand, name = "unpack")]
struct UnpackCommand {
    /// path to the .cmp file
    #[argh(positional)]
    path: PathBuf,
}

/// packs files into a .cmp file
#[derive(FromArgs)]
#[argh(subcommand, name = "pack")]
struct PackCommand {
    /// output file (default: packed.cmp)
    #[argh(option, short = 'o')]
    output: Option<PathBuf>,
}

fn main() -> Result<()> {
    let args = argh::from_env::<Args>();

    let Some(command) = args.command else {
        return run_editor();
    };

    let current_dir = std::env::current_dir()?;
    match command {
        SubCommand::Pack(pack) => {
            crate::types::packed::pack(&current_dir, pack.output.unwrap_or("packed.cmp".into()))
        }
        SubCommand::Unpack(unpack) => crate::types::packed::unpack(&unpack.path, &current_dir),
    }
}
