#![feature(round_char_boundary)]

use std::{
    env,
    fs,
    io::{self, BufRead, ErrorKind},
    path::{Path, PathBuf},
};

use anyhow::Context;
use chrono::{prelude::*, NaiveDateTime};

use clap::ArgMatches;
use today::{
    combine,
    formatter::{self, Cell, Field, ListFormatter, TodayFormatter, Visibility},
    monoid::{Last, Monoid},
    parser::program::{Parser, Program},
    partial_config::{Build, Run, Select},
    repository::Repository,
    semigroup::Semigroup,
    Task,
    TaskList,
    TaskName,
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

fn read_args(args: &ArgMatches) -> AppPaths<Build> {
    if let Some((subcommand, matches)) = args.subcommand() {
        let command = subcommand
            .parse::<Command>()
            .expect("Parsing command failed")
            .into();
        let detached = matches
            .try_contains_id("detached")
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

fn main() -> anyhow::Result<()> {
    let matches = cli::options();

    let config = combine! {
        AppPaths::empty() =>
            read_xdg().unwrap_or_default(),
            read_env().unwrap_or_default(),
            read_args(&matches)
    }
    .build();

    let mut path = config.data.value().to_owned();
    path.push("today.json");
    let mut json = today::json::JsonRepository::new(&path);

    let mut tasks = TaskList::new();
    tasks.extend(json.all().map(|x| x.to_vec())?);

    let old_conf = config.clone();
    let app = app::App::new(config, json);

    match matches.subcommand() {
        Some(("list", _sub_matches)) => {
            let shortest_id = commands::shortest_id_length(tasks.as_slice()).max(5);
            let mut formatter = ListFormatter::new();

            let default_cell = Cell::default().with_margin((0, 1));
            formatter.insert(
                Field::Id,
                default_cell
                    .clone()
                    .with_size(formatter::Size::Max(shortest_id)),
            );
            formatter.insert(Field::Name, default_cell.clone());
            formatter.insert(Field::Time, default_cell);

            commands::list(&tasks, &formatter)?;
        }
        Some(("today", _sub_matches)) => {
            let mut formatter = TodayFormatter::new();
            formatter.insert(
                Field::Id,
                Cell::default().with_visibility(Visibility::Hidden),
            );
            commands::today(&mut tasks, &formatter)?;
        }
        Some(("remove", sub_matches)) => {
            let id = sub_matches.value_of("id").unwrap();
            cli::remove(id, &mut tasks)?;
        }
        Some(("add", sub_matches)) => {
            let name = sub_matches
                .value_of("name")
                .and_then(TaskName::new)
                .or_else(|| TaskName::new(&ui::prompt_name().ok()?))
                .unwrap_or_else(|| {
                    panic!(
                        "I expected a valid TaskName but could not construct one from the type \
                         {:?}",
                        sub_matches.value_of("name")
                    )
                });

            let due = if sub_matches.is_present("now") {
                None
            } else {
                sub_matches
                    .value_of("due")
                    .map(|x| NaiveDateTime::parse_from_str(x, "%Y-%m-%d %H:%M").unwrap())
                    .map(|x| Utc.from_local_datetime(&x).unwrap())
                    .or_else(|| {
                        let date = ui::prompt_due().ok()?;

                        if let Some(date) = date {
                            let time = ui::prompt_time().ok()?;
                            let due = date.and_time(time);
                            Some(Utc.from_local_datetime(&due).unwrap())
                        } else {
                            None
                        }
                    })
            };

            cli::add(name, due, &mut tasks)?;
        }
        Some(("edit", _sub_matches)) =>
        {
            #[allow(clippy::significant_drop_in_scrutinee)]
            for line in io::stdin().lock().lines() {
                let line = line?;
                let mut parser = Parser::new(&line);
                let program = parser.parse()?;
                match program {
                    Program::Edit { id, name, due } => {
                        let filtered_tasks = tasks
                            .iter()
                            .filter(|x| x.id().to_string().starts_with(&id))
                            .collect::<Vec<_>>();

                        match filtered_tasks.len() {
                            0 => eprintln!("No task found with the id '{}'", id),
                            1 => {
                                let new_task =
                                    filtered_tasks[0].clone().with_name(name).with_due(due);
                                if let Err(e) = tasks.edit(new_task) {
                                    eprintln!("Unable to edit the task: {e}");
                                }
                            }
                            _ => eprintln!(
                                "More than one possible task was found with the id '{}'",
                                id
                            ),
                        }
                    }
                    Program::Add(task) => tasks.add(task),
                    Program::Remove(partial_id) => cli::remove(&partial_id, &mut tasks)?,
                    _ => {}
                };
            }
        }
        Some(("config", _sub_matches)) => println!("{old_conf}"),
        _ => {
            interactive(&mut tasks)?;
        }
    }

    let db = tasks.iter().collect::<Vec<_>>();
    save_tasks(&db, &path).context(format!(
        "Could not save to file '{}'",
        path.to_str().unwrap_or_default()
    ))?;

    Ok(())
}

fn interactive(tasks: &mut TaskList) -> anyhow::Result<()> {
    let mut formatter = TodayFormatter::new();
    formatter.insert(
        Field::Id,
        Cell::default().with_visibility(Visibility::Hidden),
    );
    formatter.insert(Field::Name, Cell::default().with_margin((0, 1)));

    let mut formatter = ListFormatter::new();

    let default_cell = Cell::default().with_margin((0, 1));
    formatter.insert(
        Field::Id,
        default_cell.clone().with_visibility(Visibility::Hidden),
    );
    formatter.insert(Field::Name, default_cell.clone());
    formatter.insert(Field::Time, default_cell.clone());

    loop {
        let option = ui::menu()?;

        match option {
            ui::MenuOption::Quit => break,
            ui::MenuOption::Add => commands::add(ui::prompt_task, tasks)?,
            ui::MenuOption::List => {
                let max_name_length = tasks
                    .iter()
                    .map(|x| x.name().len())
                    .max()
                    .unwrap_or_default();

                let col = formatter
                    .column(Field::Name)
                    .or_insert_with(|| default_cell.clone().into());
                *col = Cell::default()
                    .with_size(formatter::Size::Max(max_name_length))
                    .into();

                commands::list(tasks, &formatter)?
            }
            ui::MenuOption::Remove => commands::remove(ui::prompt_task_remove, tasks)?,
            ui::MenuOption::Today => commands::today(tasks, &formatter)?,
        }
    }

    Ok(())
}

fn save_tasks<P: AsRef<Path>>(tasks: &[&Task], path: P) -> anyhow::Result<()> {
    let json = serde_json::to_string(tasks)?;
    let directory = path
        .as_ref()
        .parent()
        .expect("Expected a directory for the file");

    if let Err(err) = fs::metadata(directory) {
        if err.kind() == ErrorKind::NotFound {
            fs::create_dir_all(directory).context(format!(
                "Could not create directory '{}'",
                directory.to_str().unwrap_or_default()
            ))?
        } else {
            anyhow::bail!(err)
        }
    }

    fs::write(path, &json)?;
    Ok(())
}
