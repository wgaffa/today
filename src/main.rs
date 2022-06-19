#![feature(round_char_boundary)]

use std::{
    env,
    io,
    path::PathBuf,
};

use clap::ArgMatches;
use today::{
    combine,
    monoid::{Last, Monoid},
    partial_config::{Build, Run, Select},
    semigroup::Semigroup, parser::program::Parser,
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
        detached: Last<bool> => bool,
        command: Last<Command> => Command,
    }
);

impl AppPaths<Build> {
    pub fn build(self) -> AppPaths<Run> {
        AppPaths {
            config: self.config.get().0.unwrap_or_default().into(),
            data: self.data.get().0.unwrap_or_default().into(),
            detached: self.detached.get().0.unwrap_or_default().into(),
            command: self.command.get().0.unwrap_or_default().into(),
        }
    }
}

impl AppPaths<Run> {
    pub fn unbuild(self) -> AppPaths<Build> {
        AppPaths {
            config: self.config.into(),
            data: self.data.into(),
            detached: self.detached.into(),
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

today::semigroup_default!(AppPaths<Build>: config, data, detached, command);
today::monoid_default!(AppPaths<Build>: config, data, detached, command);

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
        let detached = matches
            .try_contains_id("detached")
            .unwrap_or_default()
            .into();

        let command = parse_command(&subcommand, matches)
            .unwrap_or_default()
            .into();
        AppPaths {
            detached,
            command,
            ..Default::default()
        }
    } else {
        Default::default()
    }
}

fn parse_command(command: &str, mut matches: ArgMatches) -> Option<Command> {
    match command {
        "add" => {
            let name = matches.try_remove_one("name").ok().flatten();
            let now = matches.try_contains_id("now")
                .ok()
                .and_then(|_| None);
            let due = matches.remove_one("due")
                .and_then(|x| Some(x));

            let due = now.or(due);
            Some(Command::Add{ name, due })
        }
        "list" => Some(Command::List),
        "remove" => {
            let id = matches.remove_one::<String>("id").unwrap();
            Some(Command::Remove(id))
        }
        "today" => Some(Command::Today),
        "edit" => {
            let edits = io::stdin().lines().map(|line| {
                let line = line.unwrap();
                let mut parser = Parser::new(&line);
                parser.parse()
            })
            .inspect(|result| {
                if let Err(e) = result {
                    eprintln!("Failed to parse: {e}");
                }
            })
            .flatten()
            .collect::<Vec<_>>();

            Some(Command::Edit { program: edits })
        }
        _ => None,
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

    let mut app = app::App::new(config, json);

    app.run()
}

