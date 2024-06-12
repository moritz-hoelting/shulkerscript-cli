use std::{path::Path, thread, time::Duration};

use clap::Subcommand;
use notify_debouncer_mini::{new_debouncer, notify::*, DebounceEventResult};

use super::BuildArgs;
use crate::{
    cli::Command,
    error::Result,
    terminal_output::{print_error, print_info},
};

#[derive(Debug, clap::Args, Clone)]
pub struct WatchArgs {
    /// Do not run the command when starting, only after changes are detected.
    #[clap(short, long)]
    no_inital: bool,
    /// The time to wait in ms before running the command after changes are detected.
    #[clap(short, long, default_value = "2000")]
    debounce_time: u64,
    /// The command to run when changes are detected.
    #[command(subcommand)]
    cmd: Option<WatchSubcommand>,
}

#[derive(Debug, Clone, Subcommand)]
pub enum WatchSubcommand {
    /// Build the project.
    Build(BuildArgs),
}

impl From<WatchSubcommand> for Command {
    fn from(value: WatchSubcommand) -> Self {
        match value {
            WatchSubcommand::Build(args) => Command::Build(args),
        }
    }
}

pub fn watch(verbose: bool, args: &WatchArgs) -> Result<()> {
    let cmd = Command::from(
        args.cmd
            .to_owned()
            .unwrap_or_else(|| WatchSubcommand::Build(BuildArgs::default())),
    );

    let project_path = match &args.cmd {
        Some(WatchSubcommand::Build(args)) => args.path.as_path(),
        None => Path::new("."),
    };

    #[allow(clippy::collapsible_if)]
    if !args.no_inital {
        if cmd.run(verbose).is_err() {
            print_error("Command failed to run initially");
        }
    }

    ctrlc::set_handler(move || {
        print_info("Stopping watcher...");
        std::process::exit(0);
    })
    .expect("Error setting Ctrl-C handler");

    let mut debouncer = new_debouncer(
        Duration::from_millis(args.debounce_time),
        move |res: DebounceEventResult| {
            if res.is_ok() {
                if cmd.run(verbose).is_err() {
                    print_error("Command failed to run");
                }
            } else {
                std::process::exit(1);
            }
        },
    )
    .expect("Failed to initialize watcher");

    let watcher = debouncer.watcher();
    watcher
        .watch(project_path.join("src").as_path(), RecursiveMode::Recursive)
        .expect("Failed to watch project src");
    watcher
        .watch(
            project_path.join("pack.png").as_path(),
            RecursiveMode::NonRecursive,
        )
        .expect("Failed to watch project pack.png");
    watcher
        .watch(
            project_path.join("pack.toml").as_path(),
            RecursiveMode::NonRecursive,
        )
        .expect("Failed to watch project pack.toml");

    loop {
        thread::sleep(Duration::from_secs(60));
    }
}
