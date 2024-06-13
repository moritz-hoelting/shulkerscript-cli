use std::{
    env, io, iter,
    path::PathBuf,
    process::{self, ExitStatus},
    thread,
    time::Duration,
};

use clap::Parser;
use notify_debouncer_mini::{new_debouncer, notify::*, DebounceEventResult};

use crate::{
    cli::Args,
    error::Result,
    terminal_output::{print_error, print_info, print_warning},
};

#[derive(Debug, clap::Args, Clone)]
pub struct WatchArgs {
    /// The path of the project to watch.
    #[clap(default_value = ".")]
    pub path: PathBuf,
    /// Do not run the command when starting, only after changes are detected.
    #[clap(short, long)]
    pub no_inital: bool,
    /// The time to wait in ms before running the command after changes are detected.
    #[clap(short, long, default_value = "2000")]
    pub debounce_time: u64,
    /// The commands to run in the project directory when changes are detected.
    #[clap(short = 'x', long, default_value = "build .")]
    pub execute: Vec<String>,
}

#[derive(Debug, Clone)]
enum WatchCommand {
    Internal(Args),
    External(String),
}

pub fn watch(_verbose: bool, args: &WatchArgs) -> Result<()> {
    print_info(format!("Watching project at {}", args.path.display()));

    let commands = args
        .execute
        .iter()
        .map(|cmd| {
            let split = cmd.split_whitespace();
            let prog_name = std::env::args()
                .next()
                .unwrap_or(env!("CARGO_PKG_NAME").to_string());
            if let Ok(args) =
                Args::try_parse_from(iter::once(prog_name.as_str()).chain(split.clone()))
            {
                WatchCommand::Internal(args)
            } else {
                WatchCommand::External(cmd.to_owned())
            }
        })
        .collect::<Vec<_>>();

    if env::set_current_dir(args.path.as_path()).is_err() {
        print_warning("Failed to change working directory to project path. Commands may not work.");
    }

    #[allow(clippy::collapsible_if)]
    if !args.no_inital {
        run_cmds(&commands, true);
    }

    ctrlc::set_handler(move || {
        print_info("Stopping watcher...");
        process::exit(0);
    })
    .expect("Error setting Ctrl-C handler");

    let mut debouncer = new_debouncer(
        Duration::from_millis(args.debounce_time),
        move |res: DebounceEventResult| {
            if res.is_ok() {
                run_cmds(&commands, false)
            } else {
                process::exit(1);
            }
        },
    )
    .expect("Failed to initialize watcher");

    let watcher = debouncer.watcher();
    watcher
        .watch(args.path.join("src").as_path(), RecursiveMode::Recursive)
        .expect("Failed to watch project src");
    watcher
        .watch(
            args.path.join("pack.png").as_path(),
            RecursiveMode::NonRecursive,
        )
        .expect("Failed to watch project pack.png");
    watcher
        .watch(
            args.path.join("pack.toml").as_path(),
            RecursiveMode::NonRecursive,
        )
        .expect("Failed to watch project pack.toml");

    loop {
        thread::sleep(Duration::from_secs(60));
    }
}

fn run_cmds(cmds: &[WatchCommand], initial: bool) {
    if initial {
        print_info("Running commands initially...");
    } else {
        print_info("Changes have been detected. Running commands...");
    }
    for (index, cmd) in cmds.iter().enumerate() {
        match cmd {
            WatchCommand::Internal(args) => {
                if args.run().is_err() {
                    print_error(format!("Error running command: {}", index + 1));
                    print_error("Not running further commands.");
                    break;
                }
            }
            WatchCommand::External(cmd) => {
                let status = run_shell_cmd(cmd);
                match status {
                    Ok(status) if !status.success() => {
                        print_error(format!(
                            "Command {} exited unsuccessfully with status code {}",
                            index + 1,
                            status.code().unwrap_or(1)
                        ));
                        print_error("Not running further commands.");
                        break;
                    }
                    Ok(_) => {}
                    Err(_) => {
                        print_error(format!("Error running command: {}", index + 1));
                        print_error("Not running further commands.");
                        break;
                    }
                }
            }
        }
    }
}

fn run_shell_cmd(cmd: &str) -> io::Result<ExitStatus> {
    let mut command = if cfg!(target_os = "windows") {
        let mut command = process::Command::new("cmd");
        command.arg("/C");
        command
    } else {
        let mut command = process::Command::new(env::var("SHELL").unwrap_or("sh".to_string()));
        command.arg("-c");
        command
    };

    command.arg(cmd).status()
}
