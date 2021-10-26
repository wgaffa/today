use std::{
    collections::HashMap,
    env,
    fs,
    io::{self, ErrorKind},
    path::{Path, PathBuf},
};
use anyhow::{anyhow, Error};

use today::{TaskManager, combine, partial_config::{Last, Semigroup, Monoid}};
use today_derive::*;

mod ui;

#[derive(Debug, Semigroup, Monoid, Default)]
struct AppPaths {
    config: Last<PathBuf>,
    data: Last<PathBuf>,
}

fn read_env() -> anyhow::Result<AppPaths> {
    env::var("TODAY_CONFIG_PATH")
        .map_err(|e| anyhow::Error::new(e))
        .and_then(|x| {
            match fs::metadata(&x) {
                Ok(_meta) => Ok(
                    AppPaths {
                        config: PathBuf::from(x).into(),
                        ..Default::default()
                    }
                ),
                Err(e) => Err(Error::new(e)),
            }
        })
}

fn read_xdg() -> anyhow::Result<AppPaths> {
    let path = config_path().ok_or(anyhow!("Failed to get default XDG config path"))?;
    match setup_config(&path) {
        Ok(()) => Ok(
            AppPaths {
                config: path.into(),
                ..Default::default()
            }
        ),
        Err(e) => Err(Error::new(e)),
    }
}

fn main() -> anyhow::Result<()> {
    let config = combine!{
        AppPaths::default() =>
            read_env().unwrap_or_default(),
            read_xdg().unwrap_or_default(),
    };
    println!("{:?}", config);

    let mut tasks = TaskManager::new();
    let mut dispatcher: HashMap<ui::MenuOption, fn(&mut TaskManager) -> anyhow::Result<()>> = HashMap::new();
    dispatcher.insert(ui::MenuOption::Add, |tasks| {
        let task = ui::prompt_task()?;
        tasks.add(task);
        Ok(())
    });
    dispatcher.insert(ui::MenuOption::List, |tasks| {
        println!("{:#?}", tasks);
        Ok(())
    });
    dispatcher.insert(ui::MenuOption::Quit, |_| Ok(()));

    loop {
        let option = ui::menu()?;
        let callback = dispatcher.get_mut(&option).unwrap();
        callback(&mut tasks)?;

        if option == ui::MenuOption::Quit {
            break;
        }
    }

    Ok(())
}

fn config_path() -> Option<PathBuf> {
    let config_path = dirs::config_dir();

    const APP_ID: &str = "today";
    config_path.and_then(|mut x| {
        x.push(APP_ID);
        Some(x)
    })
}

fn setup_config<P: AsRef<Path>>(path: P) -> Result<(), io::Error> {
    match fs::metadata(&path) {
        Err(e) if e.kind() == ErrorKind::NotFound => {
            fs::create_dir_all(&path)
        },
        Ok(meta) => {
            if meta.is_dir() {
                Ok(())
            } else {
                todo!()
            }
        }
        Err(e) => Err(e),
    }
}
