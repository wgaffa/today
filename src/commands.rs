use itertools::Itertools;

use today::{formatter::TaskFormatter, Task, TaskList};

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

pub fn list<T: TaskFormatter>(tasks: &TaskList, f: &T) -> anyhow::Result<()> {
    let mut tasks = tasks.iter().collect::<Vec<_>>();
    tasks.sort_by(|&x, &y| x.due().cmp(&y.due()));

    for task in tasks {
        println!("{}", <T as TaskFormatter>::format(f, task));
    }

    Ok(())
}

pub fn shortest_id_length(tasks: &[Task]) -> usize {
    if tasks.len() < 1 {
        return 0;
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
    count
}

pub fn today<T: TaskFormatter>(tasks: &mut TaskList, f: &T) -> anyhow::Result<()> {
    for task in tasks.today() {
        println!("{}", <T as TaskFormatter>::format(f, task));
    }

    Ok(())
}
