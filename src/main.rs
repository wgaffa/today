use std::{
    env,
    fs,
    io::ErrorKind,
    path::{Path, PathBuf},
};

use anyhow::Context;

use today::{
    combine,
    formatter::{self, Cell, Field, ListFormatter, TodayFormatter, Visibility},
    monoid::{Last, Monoid},
    partial_config::{Build, Run, Select},
    semigroup::Semigroup,
    Task,
    TaskList,
};

mod cli;
mod commands;
mod ui;

today::config!(
    derive(Debug, Default)
    AppPaths {
        config: Last<PathBuf> => PathBuf,
        data: Last<PathBuf> => PathBuf,
    }
);

impl AppPaths<Build> {
    pub fn build(self) -> AppPaths<Run> {
        AppPaths {
            config: self.config.get().0.unwrap_or_default().into(),
            data: self.data.get().0.unwrap_or_default().into(),
        }
    }
}

impl AppPaths<Run> {
    pub fn unbuild(self) -> AppPaths<Build> {
        AppPaths {
            config: self.config.into(),
            data: self.data.into(),
        }
    }
}

today::semigroup_default!(AppPaths<Build>: config, data);
today::monoid_default!(AppPaths<Build>: config, data);

macro_rules! env_paths {
    ($t:ident , $($i:ident as $e:expr => $f:expr),* $(,)?) => {
        $t {
            $(
                $i: env::var($e).ok().map($f).unwrap_or_default().into(),
            )*
        }
    };
}

fn read_env() -> anyhow::Result<AppPaths<Build>> {
    Ok(env_paths! {
        AppPaths,
        config as "TODAY_CONFIG_PATH" => |x| Last::from(PathBuf::from(x)),
        data as "TODAY_DATA_PATH" => |x| Last::from(PathBuf::from(x)),
    })
}

macro_rules! xdg_paths {
    ($t:ident , $($i:ident as $e:expr => $f:expr),* $(,)?) => (
        $t {
            $(
                $i: $e.map($f).unwrap_or_default().into(),
            )*
        }
    )
}

fn read_xdg() -> anyhow::Result<AppPaths<Build>> {
    let push_app_id = |mut x: PathBuf| {
        x.push("today");
        x
    };
    Ok(xdg_paths! {
        AppPaths,
        config as dirs::config_dir() => |x| Last::from(push_app_id(x)),
        data as dirs::data_dir() => |x| Last::from(push_app_id(x)),
    })
}

fn main() -> anyhow::Result<()> {
    let matches = cli::options();

    let config = combine! {
        AppPaths::empty() =>
            read_xdg().unwrap_or_default(),
            read_env().unwrap_or_default(),
    }
    .build();

    let mut tasks = load_tasks(&config)?;
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
            formatter.insert(Field::Time, default_cell.clone());

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
        _ => {
            interactive(&mut tasks)?;
        }
    }

    let mut task_path = config.data.value().to_owned();
    task_path.push("tasks.json");

    let db = tasks.iter().collect::<Vec<_>>();
    save_tasks(&db, &task_path).context(format!(
        "Could not save to file '{}'",
        task_path.to_str().unwrap_or_default()
    ))?;

    Ok(())
}

fn load_tasks(config: &AppPaths<Run>) -> anyhow::Result<TaskList> {
    let mut task_path = config.data.value().to_owned();
    task_path.push("tasks.json");

    let file_content = match fs::read_to_string(&task_path) {
        Err(err) if err.kind() == ErrorKind::NotFound => Ok(String::from("[]")),
        Err(err) => Err(err).context(format!(
            "Could not open file '{}'",
            task_path.to_str().unwrap_or_default()
        )),
        Ok(ok) => Ok(ok), // This basicly just converts from Result<T, io::Error> to an anyhow Result
    }?;

    let db = serde_json::from_str::<Vec<Task>>(&file_content)?;
    let mut tasks = TaskList::new();
    tasks.add_range(&db);

    Ok(tasks)
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
