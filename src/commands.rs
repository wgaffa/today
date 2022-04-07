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
    let options = tasks.iter().cloned().collect::<Vec<_>>();

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
    if tasks.len() <= 1 {
        return min_length.max(1);
    }

    let ids = tasks
        .iter()
        .map(|x| x.id().as_ref().to_simple_ref().to_string())
        .sorted()
        .collect::<Vec<_>>();

    let mut id_iter = ids.windows(2);

    let mut prefixes = Vec::new();
    let mut temp_prefix = "";
    if let Some(first) = id_iter.next() {
        for (i, (a, b)) in first[0].chars().zip(first[1].chars()).enumerate() {
            if a != b {
                prefixes.push(&first[0][..=i]);
                temp_prefix = &first[1][..=i];
                break;
            }
        }
    }

    for pair in id_iter {
        for (i, (a, b)) in pair[0].chars().zip(pair[1].chars()).enumerate() {
            if a != b {
                let new_prefix = &pair[0][..=i];

                if temp_prefix.len() > new_prefix.len() {
                    prefixes.push(temp_prefix);
                } else {
                    prefixes.push(new_prefix);
                }

                temp_prefix = &pair[1][..=i];
                break;
            }
        }
    }

    for (i, (a, b)) in ids[ids.len() - 2]
        .chars()
        .zip(ids[ids.len() - 1].chars())
        .enumerate()
    {
        if a != b {
            prefixes.push(&ids[ids.len() - 1][..=i]);
            break;
        }
    }

    let count = prefixes.iter().map(|x| x.len()).max().unwrap();
    min_length.max(count)
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
