use std::{
    env, io, iter,
    path::PathBuf,
    process::{self, ExitStatus},
    thread,
    time::Duration,
};

use clap::Parser;
use colored::Colorize;
use notify_debouncer_mini::{new_debouncer, notify::*, DebounceEventResult};

use crate::{
    cli::Args,
    error::Result,
    terminal_output::{print_error, print_info, print_warning},
    util,
};

#[derive(Debug, clap::Args, Clone)]
pub struct WatchArgs {
    /// The path of the project to watch.
    #[arg(default_value = ".")]
    pub path: PathBuf,
    /// Only run after changes are detected.
    ///
    /// Skips the initial run of the commands.
    #[arg(short, long)]
    pub no_inital: bool,
    /// The time to wait in ms before running the command after changes are detected.
    #[arg(short, long, value_name = "TIME_IN_MS", default_value = "2000")]
    pub debounce_time: u64,
    /// Additional paths to watch for changes.
    ///
    /// By default, the `src` directory, `pack.png`, and `pack.toml` as well as the defined
    /// assets directory in the config are watched.
    #[arg(short, long, value_name = "PATH")]
    pub watch: Vec<PathBuf>,
    /// The shulkerscript commands to run in the project directory when changes are detected.
    ///
    /// Use multiple times to run multiple commands.
    /// Internal commands will always run before shell commands and a command will only run if the
    /// previous one exited successfully.
    ///
    /// Use the `--no-execute` flag to disable running these commands, useful when only wanting to
    /// run shell commands and not default build command.
    #[arg(short = 'x', long, value_name = "COMMAND", default_value = "build .")]
    pub execute: Vec<String>,
    /// Do not run the internal shulkerscript commands specified by `--execute` (and the default one).
    #[arg(short = 'X', long)]
    pub no_execute: bool,
    /// The shell commands to run in the project directory when changes are detected.
    ///
    /// Use multiple times to run multiple commands.
    /// Shell commands will always run after shulkerscript commands and a command will only run
    /// if the previous one exited successfully.
    #[arg(short, long, value_name = "COMMAND")]
    pub shell: Vec<String>,
}

pub fn watch(args: &WatchArgs) -> Result<()> {
    let path = util::get_project_path(&args.path).unwrap_or(args.path.clone());
    print_info(format!("Watching project at {}", path.display()));
    print_info(format!(
        "Press {} to stop watching",
        "Ctrl-C".underline().blue()
    ));

    let commands = args
        .execute
        .iter()
        .map(|cmd| {
            let split = cmd.split_whitespace();
            let prog_name = std::env::args()
                .next()
                .unwrap_or(env!("CARGO_PKG_NAME").to_string());
            Args::parse_from(iter::once(prog_name.as_str()).chain(split.clone()))
        })
        .collect::<Vec<_>>();

    let current_dir = if args.no_inital {
        print_info("Skipping initial commands because of cli flag.");
        None
    } else {
        env::current_dir().ok()
    };

    if !args.no_inital && (current_dir.is_none() || env::set_current_dir(&path).is_err()) {
        print_warning("Failed to change working directory to project path. Commands may not work.");
    }

    #[allow(clippy::collapsible_if)]
    if !args.no_inital {
        run_cmds(&commands, args.no_execute, &args.shell, true);
    }

    ctrlc::set_handler(move || {
        print_info("Stopping watcher...");
        process::exit(0);
    })
    .expect("Error setting Ctrl-C handler");

    let shell_commands = args.shell.clone();
    let no_execute = args.no_execute;

    let mut debouncer = new_debouncer(
        Duration::from_millis(args.debounce_time),
        move |res: DebounceEventResult| {
            if res.is_ok() {
                run_cmds(&commands, no_execute, &shell_commands, false)
            } else {
                process::exit(1);
            }
        },
    )
    .expect("Failed to initialize watcher");

    if let Some(prev_cwd) = current_dir {
        env::set_current_dir(prev_cwd).expect("Failed to change working directory back");
    }

    let assets_path = super::build::get_pack_config(&path)
        .ok()
        .and_then(|(conf, _)| conf.compiler.and_then(|c| c.assets));

    let watcher = debouncer.watcher();
    watcher
        .watch(path.join("src").as_path(), RecursiveMode::Recursive)
        .expect("Failed to watch project src");
    watcher
        .watch(path.join("pack.png").as_path(), RecursiveMode::NonRecursive)
        .expect("Failed to watch project pack.png");
    watcher
        .watch(
            path.join("pack.toml").as_path(),
            RecursiveMode::NonRecursive,
        )
        .expect("Failed to watch project pack.toml");
    if let Some(assets_path) = assets_path {
        let full_assets_path = path.join(assets_path);
        if full_assets_path.exists() {
            watcher
                .watch(full_assets_path.as_path(), RecursiveMode::Recursive)
                .expect("Failed to watch project assets");
        }
    }

    // custom watch paths
    for path in args.watch.iter() {
        if path.exists() {
            watcher
                .watch(path, RecursiveMode::Recursive)
                .expect("Failed to watch custom path");
        } else {
            print_warning(format!(
                "Path {} does not exist. Skipping...",
                path.display()
            ));
        }
    }

    if env::set_current_dir(path).is_err() {
        print_warning("Failed to change working directory to project path. Commands may not work.");
    }

    loop {
        thread::sleep(Duration::from_secs(60));
    }
}

fn run_cmds(cmds: &[Args], no_execute: bool, shell_cmds: &[String], initial: bool) {
    if initial {
        print_info("Running commands initially...");
    } else {
        print_info("Changes have been detected. Running commands...");
    }
    if !no_execute {
        for (index, args) in cmds.iter().enumerate() {
            if args.run().is_err() {
                print_error(format!("Error running command: {}", index + 1));
                print_error("Not running further commands.");
                return;
            }
        }
    }
    for (index, cmd) in shell_cmds.iter().enumerate() {
        let status = run_shell_cmd(cmd);
        match status {
            Ok(status) if !status.success() => {
                print_error(format!(
                    "Shell command {} exited unsuccessfully with status code {}",
                    index + 1,
                    status.code().unwrap_or(1)
                ));
                print_error("Not running further shell commands.");
                return;
            }
            Ok(_) => {}
            Err(_) => {
                print_error(format!("Error running shell command: {}", index + 1));
                print_error("Not running further shell commands.");
                return;
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
