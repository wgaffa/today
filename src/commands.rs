use std::borrow::Borrow;

use termion::color;

use crate::{TaskList, Task};

pub fn add<F>(input: F, tasks: &mut TaskList) -> anyhow::Result<()>
where
    F: Fn() -> anyhow::Result<Task>
{
    let task = input()?;
    tasks.add(task);
    Ok(())
}

pub fn remove<F>(input: F, tasks: &mut TaskList) -> anyhow::Result<()>
where
    F: Fn(&[String]) -> anyhow::Result<Option<String>>
{
    let options = tasks
        .iter()
        .map(|x| x.name().to_owned())
        .collect::<Vec<_>>();

    if !options.is_empty() {
        let task = input(&options)?;
        if let Some(task) = task {
            tasks.remove(&task);
        }
    }

    Ok(())
}

pub fn list(tasks: &mut TaskList) -> anyhow::Result<()> {
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

pub fn list_with_ids(tasks: &TaskList) -> anyhow::Result<()> {
    let mut tasks = tasks.iter().collect::<Vec<_>>();
    tasks.sort_by(|&x, &y| x.due().cmp(&y.due()));
    let length = tasks.iter().map(|&x| x.name().len()).max();
    if let Some(length) = length {
        for task in tasks {
            let due = task.due().map_or(String::from("ASAP"), |x| {
                x.format("%Y-%m-%d %H:%M").to_string()
            });
            let id: &uuid::Uuid = task.id().borrow();
            println!(
                "{id} {name:width$} {due}",
                name = task.name(),
                due = due,
                width = length,
            );
        }
    }

    Ok(())
}

pub fn today(tasks: &mut TaskList) -> anyhow::Result<()> {
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

