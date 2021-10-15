use std::{collections::HashMap, error::Error};

use today::TaskManager;

mod ui;

fn main() -> Result<(), Box<dyn Error>>{
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
