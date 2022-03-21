use termion::color;

use crate::TaskManager;
use crate::ui;

pub fn add(tasks: &mut TaskManager) -> anyhow::Result<()> {
    let task = ui::prompt_task()?;
    tasks.add(task);
    Ok(())
}

pub fn remove(tasks: &mut TaskManager) -> anyhow::Result<()> {
    let options = tasks
        .iter()
        .map(|x| x.name().to_owned())
        .collect::<Vec<_>>();

    if !options.is_empty() {
        let task = ui::prompt_task_remove(&options)?;
        if let Some(task) = task {
            tasks.remove(&task);
        }
    }

    Ok(())
}

pub fn list(tasks: &mut TaskManager) -> anyhow::Result<()> {
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
}

pub fn today(tasks: &mut TaskManager) -> anyhow::Result<()> {
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
}

