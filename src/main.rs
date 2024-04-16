use clap::Parser;
use color_eyre::eyre::Result;
use shulkerscript::cli::Args;

fn main() -> Result<()> {
    color_eyre::install()?;
    let args = Args::parse();

    args.run()?;

    Ok(())
}
