use std::{
    collections::HashMap,
    env::{self, VarError},
    error::Error,
    fs,
    io::{self, ErrorKind},
    path::{Path, PathBuf}
};

use today::TaskManager;

mod ui;

fn main() -> Result<(), Box<dyn Error>>{
    let conf_path = env::var("TODAY_CONFIG_PATH")
        .and_then(|x| {
            match fs::metadata(&x) {
                Ok(_meta) => Ok(x),
                Err(_) => Err(VarError::NotPresent),
            }
        })
        .or_else(|_error| {
            if let Some(path) = config_path() {
                match setup_config(&path) {
                    Ok(()) => Ok(path.to_str().unwrap().to_owned()),
                    Err(e) => Err(e),
                }
            } else {
                todo!()
            }
        })?;
    println!("{:?}", conf_path);

    let mut tasks = TaskManager::new();
    let mut dispatcher: HashMap<ui::MenuOption, fn(&mut TaskManager) -> Result<(), Box<dyn Error>>> = HashMap::new();
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
