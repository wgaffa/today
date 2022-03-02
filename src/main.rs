use std::{
    collections::HashMap,
    env,
    fs,
    path::{Path, PathBuf},
};

use anyhow::Context;

use today::{
    combine,
    monoid::{Last, Monoid},
    partial_config::{Build, Run, Select},
    semigroup::Semigroup,
    Task,
    TaskManager,
};

use termion::color;

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
    let config = combine! {
        AppPaths::empty() =>
            read_xdg().unwrap_or_default(),
            read_env().unwrap_or_default(),
    }
    .build();
    println!("{:?}", config);

    let mut tasks = TaskManager::new();
    let mut dispatcher: HashMap<ui::MenuOption, fn(&mut TaskManager) -> anyhow::Result<()>> =
        HashMap::new();
    dispatcher.insert(ui::MenuOption::Add, |tasks| {
        let task = ui::prompt_task()?;
        tasks.add(task);
        Ok(())
    });
    dispatcher.insert(ui::MenuOption::Remove, |tasks| {
        let options = tasks
            .iter()
            .map(|x| x.name().to_owned())
            .collect::<Vec<_>>();
        let task = ui::prompt_task_remove(&options)?;
        tasks.remove(&task);
        Ok(())
    });
    dispatcher.insert(ui::MenuOption::List, |tasks| {
        let mut tasks = tasks.iter().collect::<Vec<_>>();
        tasks.sort_by(|&x, &y| x.due().cmp(&y.due()));
        let length = tasks.iter().map(|&x| x.name().len()).max();
        if let Some(length) = length {
            for task in tasks {
                let due = task.due().map_or(String::from("ASAP"), |x| {
                    x.format("%Y-%m-%d %H:%M").to_string()
                });
                println!(
                    "{name:width$} {due}",
                    name = task.name(),
                    due = due,
                    width = length,
                );
            }
        }

        Ok(())
    });
    dispatcher.insert(ui::MenuOption::Quit, |_| Ok(()));
    dispatcher.insert(ui::MenuOption::Today, |tasks| {
        for task in tasks.today() {
            let time = task.due().map_or(String::from("Now"), |x| {
                x.format("%Y-%m-%d %H:%M").to_string()
            });
            println!(
                "{}{:>16}{}: {}",
                color::Fg(color::LightRed),
                time,
                color::Fg(color::Reset),
                task.name()
            );
        }

        Ok(())
    });

    let mut task_path = config.data.value().to_owned();
    task_path.push("tasks.json");

    let file_content = fs::read_to_string(&task_path).context(format!(
        "error reading '{}'",
        task_path.to_str().unwrap_or_default()
    ))?;

    let db = serde_json::from_str::<Vec<Task>>(&file_content)?;
    tasks.add_range(&db);

    loop {
        let option = ui::menu()?;
        let callback = dispatcher.get_mut(&option).unwrap();
        callback(&mut tasks)?;

        if option == ui::MenuOption::Quit {
            let db = tasks.iter().collect::<Vec<_>>();
            save_tasks(&db, &task_path)?;
            break;
        }
    }

    Ok(())
}

fn save_tasks<P: AsRef<Path>>(tasks: &[&Task], path: P) -> anyhow::Result<()> {
    let json = serde_json::to_string(tasks)?;
    fs::write(path, &json)?;
    Ok(())
}
