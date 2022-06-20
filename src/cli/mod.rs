use chrono::prelude::*;
use clap::{command, Arg, ArgAction, ArgMatches, Command};

use today::{Task, TaskList, TaskName};

mod constants;
pub use constants::*;

pub fn options() -> ArgMatches {
    command!()
        .about("Manage tasks to do today")
        .args(&[
            Arg::new(ARG_CONFIG)
                .long("config-only")
                .action(ArgAction::SetTrue)
                .help("Print only the configuration of this run but don't run it"),
        ])
        .subcommand(Command::new(ARG_COMMAND_LIST).about("List all tasks"))
        .subcommand(
            Command::new(ARG_COMMAND_TODAY)
                .arg(
                    Arg::new(ARG_WATCH_MODE)
                        .short('w')
                        .long("watch")
                        .action(ArgAction::SetTrue)
                        .help("Run in watch mode"),
                )
                .about("List tasks that are due today"),
        )
        .subcommand(
            Command::new(ARG_COMMAND_REMOVE)
                .arg(
                    Arg::new(ARG_ID)
                        .required(true)
                        .value_name("ID")
                        .help("The id of the task to remove"),
                )
                .about("Removes a task"),
        )
        .subcommand(
            Command::new(ARG_COMMAND_ADD)
                .args(&[
                    Arg::new(ARG_NOW)
                        .short('n')
                        .long("now")
                        .conflicts_with(ARG_DUE)
                        .help("Set due to be done ASAP"),
                    Arg::new(ARG_DUE)
                        .short('d')
                        .long("due")
                        .takes_value(true)
                        .value_parser(clap::builder::ValueParser::new(|x: &str| {
                            NaiveDateTime::parse_from_str(x, "%Y-%m-%d %H:%M")
                                .or(
                                    NaiveDate::parse_from_str(x, "%Y-%m-%d")
                                        .map(|x| x.and_hms(0, 0, 0))
                                )
                                .map(|x| Utc.from_local_datetime(&x).unwrap())
                        }))
                        .help("Set the due date in the format YYYY-MM-DD HH:MM"),
                    Arg::new(ARG_NAME)
                        .required(false)
                        .value_name("NAME")
                        .validator(|x| {
                            TaskName::new(x).ok_or_else(|| {
                                anyhow::anyhow!(
                                    "A task name must have atleast one printable character"
                                )
                            })
                        })
                        .help("The task name to be done at the specified due date"),
                ])
                .about("Add a new task"),
        )
        .subcommand(Command::new(ARG_COMMAND_EDIT).about("Edit one or more tasks"))
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
