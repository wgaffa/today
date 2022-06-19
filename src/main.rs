#![feature(round_char_boundary)]

use std::{env, path::PathBuf};

use clap::ArgMatches;
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
    }
);

impl AppPaths<Build> {
    pub fn build(self) -> AppPaths<Run> {
        AppPaths {
            config: self.config.get().0.unwrap_or_default().into(),
            data: self.data.get().0.unwrap_or_default().into(),
            command: self.command.get().0.unwrap_or_default().into(),
        }
    }
}

impl AppPaths<Run> {
    pub fn unbuild(self) -> AppPaths<Build> {
        AppPaths {
            config: self.config.into(),
            data: self.data.into(),
            command: self.command.into(),
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

today::semigroup_default!(AppPaths<Build>: config, data, command);
today::monoid_default!(AppPaths<Build>: config, data, command);

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
    if let Some((subcommand, matches)) = args.remove_subcommand() {
        let command = commands::parser::parse(&subcommand, matches)
            .unwrap_or_default()
            .into();
        AppPaths {
            command,
            ..Default::default()
        }
    } else {
        Default::default()
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
    let mut file_watch = Hotwatch::new().expect("Failed to initialize a notifier");
    let detached_mode = if let Command::Today { detached: true, .. } = config.command.value() {
        true
    } else {
        false
    };

    let mut app = app::App::new(config, json);
    if detached_mode {
        let tx_file_changed = tx.clone();
        file_watch.watch(path, move |_| {
            let _ = tx_file_changed.send(());
        })?;
        app = app.with_event_file_changed(rx);
    }

    // let app_thread = thread::spawn(move || app.run());
    // let _ = app_thread.join();

    app.run()
}
