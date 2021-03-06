#![feature(round_char_boundary)]

use std::{env, path::PathBuf, thread};

use clap::ArgMatches;
use crossterm::event::{read, Event, KeyCode, KeyEvent, KeyModifiers};
use hotwatch::Hotwatch;

use today::{
    combine,
    monoid::{Last, Monoid},
    partial_config::{Build, Run, Select},
    semigroup::Semigroup,
};

mod app;
mod cli;
mod commands;
mod ui;

use commands::Command;

today::config!(
    derive(Debug, Default, Clone)
    AppPaths {
        config: Last<PathBuf> => PathBuf,
        data: Last<PathBuf> => PathBuf,
        command: Last<Command> => Command,
        watch_mode: Last<bool> => bool,
        config_only: Option<bool> => bool,
    }
);

impl AppPaths<Build> {
    pub fn build(self) -> AppPaths<Run> {
        AppPaths {
            config: self.config.get().0.unwrap_or_default().into(),
            data: self.data.get().0.unwrap_or_default().into(),
            command: self.command.get().0.unwrap_or_default().into(),
            watch_mode: self.watch_mode.get().0.unwrap_or_default().into(),
            config_only: self.config_only.get().unwrap_or_default().into(),
        }
    }
}

impl AppPaths<Run> {
    pub fn unbuild(self) -> AppPaths<Build> {
        AppPaths {
            config: self.config.into(),
            data: self.data.into(),
            command: self.command.into(),
            watch_mode: self.watch_mode.into(),
            config_only: self.config_only.into(),
        }
    }
}

impl std::fmt::Display for AppPaths<Run> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "config-dir: {}\ndata-dir: {}",
            self.config.value().to_string_lossy(),
            self.data.value().to_string_lossy()
        )
    }
}

today::semigroup_default!(AppPaths<Build>: config, data, command, watch_mode, config_only);
today::monoid_default!(AppPaths<Build>: config, data, command, watch_mode, config_only);

macro_rules! convert_env {
    ($e:expr , $f:expr) => {
        env::var($e).ok().map($f).into()
    };
}

fn read_env() -> anyhow::Result<AppPaths<Build>> {
    Ok(AppPaths {
        config: convert_env!("TODAY_CONFIG_PATH", PathBuf::from),
        data: convert_env!("TODAY_DATA_PATH", PathBuf::from),
        ..Default::default()
    })
}

macro_rules! convert_xdg {
    ($e:expr, $f:expr) => {
        $e.map($f).unwrap_or_default().into()
    };
}

fn read_xdg() -> anyhow::Result<AppPaths<Build>> {
    let push_app_id = |mut x: PathBuf| {
        x.push("today");
        x
    };
    Ok(AppPaths {
        config: convert_xdg!(dirs::config_dir(), push_app_id),
        data: convert_xdg!(dirs::data_dir(), push_app_id),
        ..Default::default()
    })
}

fn read_args(mut args: ArgMatches) -> AppPaths<Build> {
    let config_only = args.contains_id(cli::ARG_CONFIG).into();
    if let Some((subcommand, matches)) = args.remove_subcommand() {
        let watch_mode = matches
            .try_contains_id(cli::ARG_WATCH_MODE)
            .unwrap_or_default()
            .into();

        let command = commands::parser::parse(&subcommand, matches)
            .unwrap_or_default()
            .into();
        AppPaths {
            command,
            watch_mode,
            config_only,
            ..Default::default()
        }
    } else {
        AppPaths {
            config_only,
            ..Default::default()
        }
    }
}

fn main() -> anyhow::Result<()> {
    let matches = cli::options();

    let config = combine! {
        AppPaths::empty() =>
            read_xdg().unwrap_or_default(),
            read_env().unwrap_or_default(),
            read_args(matches)
    }
    .build();

    let mut path = config.data.value().to_owned();
    path.push("tasks.json");
    let json = today::json::JsonRepository::new(&path);

    let (tx, rx) = std::sync::mpsc::channel();
    let (shutdown_tx, shutdown_rx) = std::sync::mpsc::channel();
    let watch_mode = config.watch_mode.get();
    let config_only = config.config_only.get();

    let mut app = app::App::new(config, json).with_writer(std::io::stdout());

    // file_watch is declared outside of the if block because it needs to live a long time.
    // If declared inside the if block it will drop when the if block goes out of scope and
    // any watches will also drop
    let mut file_watch;
    let _input_thread;
    if watch_mode && !config_only {
        file_watch = Hotwatch::new().expect("Failed to initialize a notifier");
        let tx_file_changed = tx.clone();
        file_watch.watch(path, move |_| {
            let _ = tx_file_changed.send(());
        })?;
        app = app
            .with_event_file_changed(rx)
            .with_event_quit(shutdown_rx)
            .with_writer(ui::writers::WatchMode::new());

        let pid = nix::unistd::Pid::this();
        _input_thread = thread::spawn(move || {
            crossterm::terminal::enable_raw_mode().unwrap();

            loop {
                match read() {
                    Ok(Event::Key(KeyEvent {
                        code: KeyCode::Char('q'),
                        ..
                    })) => {
                        let _ = shutdown_tx.send(());
                        break;
                    }
                    Ok(Event::Key(KeyEvent {
                        code: KeyCode::Char('c'),
                        modifiers: KeyModifiers::CONTROL,
                    })) => {
                        let _ = nix::sys::signal::kill(pid, nix::sys::signal::Signal::SIGINT);
                    }
                    _ => {}
                }
                if let Ok(Event::Key(KeyEvent {
                    code: KeyCode::Char('q'),
                    ..
                })) = read()
                {
                    let _ = shutdown_tx.send(());
                    break;
                }
            }

            crossterm::terminal::disable_raw_mode().unwrap();
        });
    }

    app.run()
}
