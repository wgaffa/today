use itertools::Itertools;
use termion::color;

use crate::{Task, TaskList};

pub fn add<F>(input: F, tasks: &mut TaskList) -> anyhow::Result<()>
where
    F: Fn() -> anyhow::Result<Task>,
{
    let task = input()?;
    tasks.add(task);
    Ok(())
}

pub fn remove<F>(input: F, tasks: &mut TaskList) -> anyhow::Result<()>
where
    F: Fn(&[Task]) -> anyhow::Result<Option<Task>>,
{
    let options = tasks
        .iter()
        .cloned()
        .collect::<Vec<_>>();

    if !options.is_empty() {
        let task = input(&options)?;
        if let Some(task) = task {
            tasks.remove(task.id());
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
    let id_length = shortest_id_length(5, &tasks);
    if let Some(length) = length {
        for task in tasks {
            let due = task.due().map_or(String::from("ASAP"), |x| {
                x.format("%Y-%m-%d %H:%M").to_string()
            });
            let id = task
                .id()
                .as_ref()
                .to_hyphenated_ref()
                .to_string()
                .chars()
                .take(id_length)
                .collect::<String>();
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

fn shortest_id_length(min_length: usize, tasks: &[&Task]) -> usize {
    let ids = tasks
        .iter()
        .map(|x| x.id().as_ref().to_hyphenated_ref().to_string())
        .collect::<Vec<_>>();

    let columns = ids.len();
    let rows = ids[0].len();

    let mut current_length = 0;
    for row in 0..rows {
        let mut column = String::new();
        for col in 0..columns {
            column.push(
                ids[col]
                    .chars()
                    .nth(row)
                    .expect("Unexpected code point in id"),
            );
        }
        let column = column.chars().sorted().dedup().count();
        if column == columns {
            return min_length.max(current_length);
        }

        current_length += 1;
    }

    current_length
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
