use clap::{command, Arg, ArgMatches, Command};

use today::TaskList;

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
        .get_matches()
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
