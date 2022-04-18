use chrono::{prelude::*, NaiveDateTime};
use clap::{command, Arg, ArgMatches, Command};

use today::{Task, TaskList, TaskName};

pub fn options() -> ArgMatches {
    command!()
        .about("Manage tasks to do today")
        .subcommand(Command::new("list").about("List all tasks"))
        .subcommand(Command::new("today").about("List tasks that are due today"))
        .subcommand(
            Command::new("remove")
                .arg(
                    Arg::new("id")
                        .required(true)
                        .value_name("ID")
                        .help("The id of the task to remove"),
                )
                .about("Removes a task"),
        )
        .subcommand(
            Command::new("add")
                .args(&[
                    Arg::new("now")
                        .short('n')
                        .long("now")
                        .conflicts_with("due")
                        .help("Set due to be done ASAP"),
                    Arg::new("due")
                        .short('d')
                        .long("due")
                        .takes_value(true)
                        .validator(|x| NaiveDateTime::parse_from_str(x, "%Y-%m-%d %H:%M"))
                        .help("Set the due date in the format YYYY-MM-DD HH:MM"),
                    Arg::new("name")
                        .required(false)
                        .value_name("NAME")
                        .validator(|x| {
                            TaskName::new(x).ok_or_else(|| anyhow::anyhow!(
                                "A task name must have atleast one printable character"
                            ))
                        })
                        .help("The task name to be done at the specified due date"),
                ])
                .about("Add a new task"),
        )
        .subcommand(
            Command::new("edit")
                .about("Edit one or more tasks")
        )
        .get_matches()
}

pub fn add(name: TaskName, due: Option<DateTime<Utc>>, tasks: &mut TaskList) -> anyhow::Result<()> {
    let task = if let Some(date) = due {
        Task::new(name).with_date_time(date)
    } else {
        Task::new(name)
    };

    tasks.add(task);

    Ok(())
}

pub fn remove(id: &str, tasks: &mut TaskList) -> anyhow::Result<()> {
    let filtered_tasks = tasks
        .iter()
        .filter(|x| {
            let task_id = x.id().as_ref().to_simple_ref().to_string();

            task_id.starts_with(id)
        })
        .cloned()
        .collect::<Vec<_>>();

    match filtered_tasks.len() {
        0 => anyhow::bail!("No task found with that id"),
        1 => {
            let id = filtered_tasks[0].id();
            tasks.remove(id);
        }
        _ => anyhow::bail!("More than one possible task was found with that id"),
    };

    Ok(())
}
