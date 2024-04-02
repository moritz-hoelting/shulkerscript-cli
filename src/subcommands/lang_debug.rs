use clap::ValueEnum;

use crate::error::Result;
use std::path::PathBuf;

#[derive(Debug, clap::Args, Clone)]
pub struct LangDebugArgs {
    /// The path of the project to compile.
    #[clap(default_value = ".")]
    path: PathBuf,
    /// The state to dump.
    #[clap(short, long, default_value = "ast")]
    dump: DumpState,
    /// Pretty-print the output.
    #[clap(short, long)]
    pretty: bool,
}

#[derive(ValueEnum, Debug, Clone, Copy, Default)]
pub enum DumpState {
    Tokens,
    #[default]
    Ast,
    Datapack,
}

pub fn lang_debug(args: &LangDebugArgs) -> Result<()> {
    match args.dump {
        DumpState::Tokens => {
            let tokens = shulkerscript_lang::tokenize(&args.path)?;
            if args.pretty {
                println!("{:#?}", tokens);
            } else {
                println!("{:?}", tokens);
            }
        }
        DumpState::Ast => {
            let ast = shulkerscript_lang::parse(&args.path)?;
            if args.pretty {
                println!("{:#?}", ast);
            } else {
                println!("{:?}", ast);
            }
        }
        DumpState::Datapack => {
            let datapack = shulkerscript_lang::transpile(&args.path)?;
            if args.pretty {
                println!("{:#?}", datapack);
            } else {
                println!("{:?}", datapack);
            }
        }
    }
    Ok(())
}
