use std::{collections::HashMap, env, fs, io::{self, ErrorKind}, path::{Path, PathBuf}};

use today::{
    TaskManager,
    combine,
    semigroup::Semigroup,
    monoid::{Last, Monoid},
};

use today_derive::*;

mod ui;

#[derive(Debug, Semigroup, Monoid, Default)]
struct AppPaths {
    config: Last<PathBuf>,
    data: Last<PathBuf>,
}

macro_rules! env_paths {
    ($t:ident , $($i:ident as $e:expr => $f:expr),* $(,)?) => {
        $t {
            $(
                $i: env::var($e).ok().map($f).into(),
            )*
        }
    };
}

fn read_env() -> anyhow::Result<AppPaths> {
    Ok(
        env_paths! {
            AppPaths,
            config as "TODAY_CONFIG_PATH" => PathBuf::from,
            data as "TODAY_DATA_PATH" => PathBuf::from,
        }
    )
}

macro_rules! xdg_paths {
    ($t:ident , $($i:ident as $e:expr => $f:expr),* $(,)?) => (
        $t {
            $(
                $i: $e.map($f).into(),
            )*
        }
    )
}

fn read_xdg() -> anyhow::Result<AppPaths> {
    let push_app_id = |mut x: PathBuf| {x.push("today"); x};
    Ok(
        xdg_paths! {
            AppPaths,
            config as dirs::config_dir() => push_app_id,
            data as dirs::data_dir() => push_app_id,
        }
    )
}

fn main() -> anyhow::Result<()> {
    let config = combine!{
        AppPaths::empty() =>
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
