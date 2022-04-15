use std::{
    collections::HashMap,
    env,
    fs,
    io::ErrorKind,
    path::{Path, PathBuf},
};

use anyhow::Context;
use clap::{command, Arg, Command};

use today::{
    combine,
    monoid::{Last, Monoid},
    partial_config::{Build, Run, Select},
    semigroup::Semigroup,
    Task,
    TaskList, formatter::SimpleFormatter,
};

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
    let matches = command!()
        .about("Manage tasks to do today")
        .subcommand(Command::new("list").about("List all tasks"))
        .subcommand(Command::new("today").about("List tasks that are due today"))
        .subcommand(
            Command::new("remove")
                .arg(
                    Arg::new("id")
                        .required(true)
                        .value_name("ID")
                        .help("The id of the task to remove"),
                )
                .about("Removes a task"),
        )
        .get_matches();

    let config = combine! {
        AppPaths::empty() =>
            read_xdg().unwrap_or_default(),
            read_env().unwrap_or_default(),
    }
    .build();
    println!("{:?}", config);

    let mut tasks = load_tasks(&config)?;
    match matches.subcommand() {
        Some(("list", _sub_matches)) => {
            // Call a list function to list all tasks with their id's
            commands::list_with_ids(&tasks)?;
        }
        Some(("today", _sub_matches)) => {
            commands::today(&mut tasks, SimpleFormatter)?;
        }
        Some(("remove", sub_matches)) => {
            let id = sub_matches.value_of("id").unwrap();
            let filtered_tasks = tasks
                .iter()
                .filter(|x| {
                    let task_id = x
                        .id()
                        .as_ref()
                        .to_simple_ref()
                        .encode_lower(&mut uuid::Uuid::encode_buffer())
                        .to_string();

                    task_id.starts_with(id)
                })
                .cloned()
                .collect::<Vec<_>>();

            match filtered_tasks.len() {
                0 => anyhow::bail!("No task found with that id"),
                1 => {
                    let id = filtered_tasks[0].id();
                    tasks.remove(id);
                }
                _ => anyhow::bail!("More than one possible task was found with that id"),
            };
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
    let mut dispatcher: HashMap<ui::MenuOption, fn(&mut TaskList) -> anyhow::Result<()>> =
        HashMap::new();
    dispatcher.insert(ui::MenuOption::Add, |x| commands::add(ui::prompt_task, x));
    dispatcher.insert(ui::MenuOption::Remove, |x| {
        commands::remove(ui::prompt_task_remove, x)
    });
    dispatcher.insert(ui::MenuOption::List, commands::list);
    dispatcher.insert(ui::MenuOption::Quit, |_| Ok(()));
    dispatcher.insert(ui::MenuOption::Today, |x| commands::today(x, SimpleFormatter));

    loop {
        let option = ui::menu()?;
        let callback = dispatcher.get_mut(&option).unwrap();
        callback(tasks)?;

        if option == ui::MenuOption::Quit {
            break;
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
