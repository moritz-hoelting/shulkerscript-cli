use clap::ValueEnum;

use anyhow::Result;
use shulkerscript::base::FsProvider;
use std::path::PathBuf;

use crate::config::PackConfig;

#[derive(Debug, clap::Args, Clone)]
pub struct LangDebugArgs {
    /// The path of the project to compile.
    #[arg(default_value = ".")]
    pub path: PathBuf,
    /// The state to dump.
    ///
    /// Output can be the raw tokens, the abstract syntax tree, or the transpiled datapack.
    #[arg(short, long, value_name = "STATE", default_value = "ast")]
    pub dump: DumpState,
    /// Pretty-print the output.
    #[arg(short, long)]
    pub pretty: bool,
}

#[derive(ValueEnum, Debug, Clone, Copy, Default)]
pub enum DumpState {
    Tokens,
    #[default]
    Ast,
    Datapack,
}

pub fn lang_debug(args: &LangDebugArgs) -> Result<()> {
    let file_provider = FsProvider::default();
    match args.dump {
        DumpState::Tokens => {
            let tokens = shulkerscript::tokenize(&file_provider, &args.path)?;
            if args.pretty {
                println!("{:#?}", tokens);
            } else {
                println!("{:?}", tokens);
            }
        }
        DumpState::Ast => {
            let ast = shulkerscript::parse(&file_provider, &args.path)?;
            if args.pretty {
                println!("{:#?}", ast);
            } else {
                println!("{:?}", ast);
            }
        }
        DumpState::Datapack => {
            let program_paths = super::build::get_script_paths(&args.path.join("src"))?;
            let datapack = shulkerscript::transpile(
                &file_provider,
                PackConfig::DEFAULT_PACK_FORMAT,
                &program_paths,
            )?;
            if args.pretty {
                println!("{:#?}", datapack);
            } else {
                println!("{:?}", datapack);
            }
        }
    }
    Ok(())
}
