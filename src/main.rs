use std::{collections::HashMap, env, path::PathBuf};
use url::Url;

use today::{
    TaskManager,
    combine,
    semigroup::Semigroup,
    monoid::{Last, Monoid},
    partial_config::{Select, Build, Run},
};

mod ui;

today::config!(
    derive(Debug, Default)
    AppPaths {
        config: Last<PathBuf> => PathBuf,
        data: Last<PathBuf> => PathBuf,
    }
);

impl AppPaths<Build> {
    fn build(self) -> AppPaths<Run> {
        AppPaths {
            config: self.config.get().0.unwrap_or_default().into(),
            data: self.data.get().0.unwrap_or_default().into(),
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
    Ok(
        env_paths! {
            AppPaths,
            config as "TODAY_CONFIG_PATH" => |x| Last::from(PathBuf::from(x)),
            data as "TODAY_DATA_PATH" => |x| Last::from(PathBuf::from(x)),
        }
    )
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
    let push_app_id = |mut x: PathBuf| {x.push("today"); x};
    Ok(
        xdg_paths! {
            AppPaths,
            config as dirs::config_dir() => |x| Last::from(push_app_id(x)),
            data as dirs::data_dir() => |x| Last::from(push_app_id(x)),
        }
    )
}

fn main() -> anyhow::Result<()> {
    let config = combine!{
        AppPaths::empty() =>
            read_xdg().unwrap_or_default(),
            read_env().unwrap_or_default(),
    }.build();
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

