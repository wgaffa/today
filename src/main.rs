use std::{collections::HashMap, env, path::PathBuf};
use url::Url;

use today::{
    TaskManager,
    combine,
    semigroup::Semigroup,
    monoid::{Sum, Last, Monoid},
    partial_config::{Config, Build},
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
    let test: Config<Build> = Config {
        verbose: Sum::from(2).into(),
        out_file: Last::from(PathBuf::from("/home/wgaffa")).into(),
    };

    let test2: Config<Build> = Config {
        verbose: Sum::from(5).into(),
        out_file: Monoid::empty(),
    };

    let c = test.combine(test2);
    let r = c.build();

    println!("{:#?}", r);

    let config = combine!{
        AppPaths::empty() =>
            read_xdg().unwrap_or_default(),
            read_env().unwrap_or_default(),
    };
    println!("{:?}", config);

    let url = Url::parse("file:///home/wgaffa/projects/today/").unwrap();
    let path = url.to_file_path();
    println!("{:?}", path);

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
            let db = tasks.iter().collect::<Vec<_>>();
            println!("{}", ron::to_string(&db).unwrap());
            break;
        }
    }

    Ok(())
}

