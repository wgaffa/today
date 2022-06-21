use anyhow::Error;
use chrono::prelude::*;
use inquire::{
    error::{InquireError, InquireResult},
    formatter::{OptionFormatter, StringFormatter},
    ui::{RenderConfig, Styled},
    validator::StringValidator,
    CustomType,
    DateSelect,
    Select,
    Text,
};
use std::fmt::Display;

use today::{Task, TaskName};

pub mod writers;

#[non_exhaustive]
#[derive(Debug, PartialEq, Eq, Hash)]
pub enum MenuOption {
    Add,
    Remove,
    List,
    Quit,
    Today,
}

impl Display for MenuOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            MenuOption::Add => write!(f, "Add"),
            MenuOption::List => write!(f, "List"),
            MenuOption::Quit => write!(f, "Quit"),
            MenuOption::Today => write!(f, "Today"),
            MenuOption::Remove => write!(f, "Remove"),
        }
    }
}

pub fn menu() -> anyhow::Result<MenuOption> {
    let options = vec![
        MenuOption::Add,
        MenuOption::Remove,
        MenuOption::Today,
        MenuOption::List,
        MenuOption::Quit,
    ];
    let selected = Select::new("What do you wish to do?", options)
        .with_vim_mode(true)
        .prompt()?;

    Ok(selected)
}

pub fn prompt_task() -> anyhow::Result<Task> {
    let name = prompt_name()?;

    let due = prompt_due()?;

    let task = Task::new(TaskName::new(&name).unwrap());
    let task = if let Some(date) = due {
        let time = prompt_time()?;
        let due = date.and_time(time);
        task.with_date_time(Utc.from_local_datetime(&due).unwrap())
    } else {
        task
    };

    Ok(task)
}

pub fn prompt_task_remove(options: &[Task]) -> anyhow::Result<Option<Task>> {
    let formatter: OptionFormatter<Task> = &|x| x.value.name().to_owned();
    let selected = Select::new("Which task do you want to remove?", options.to_vec())
        .with_vim_mode(true)
        .with_formatter(formatter)
        .prompt_skippable()?;

    Ok(selected)
}

pub fn prompt_time() -> InquireResult<NaiveTime> {
    let time_style =
        RenderConfig::default_colored().with_canceled_prompt_indicator(Styled::new("00:00:00"));

    loop {
        let time = CustomType::<NaiveTime>::new("Time:")
            .with_default((NaiveTime::from_hms(0, 0, 0), &|x| format!("{}", x)))
            .with_placeholder("HH:MM:SS")
            .with_parser(&|x| match NaiveTime::parse_from_str(x, "%H:%M") {
                Ok(time) => Ok(time),
                Err(_) => Err(()),
            })
            .with_render_config(time_style)
            .prompt();

        match time {
            Ok(t) => return Ok(t),
            Err(InquireError::OperationCanceled) => {}
            Err(e) => panic!("Unrecoverable error: {}", e),
        }
    }
}

pub fn prompt_due() -> InquireResult<Option<NaiveDate>> {
    let date_style = RenderConfig::default_colored()
        .with_canceled_prompt_indicator(Styled::new("As soon as possible"));

    DateSelect::new("Due date:")
        .with_help_message("Press ESC to set task to be due as soon as possible")
        .with_min_date(Utc::today().naive_utc())
        .with_vim_mode(true)
        .with_render_config(date_style)
        .prompt_skippable()
}

pub fn prompt_name() -> anyhow::Result<String> {
    let name_validator: StringValidator = &|input: &str| {
        if input.chars().any(|x| !x.is_whitespace()) {
            Ok(())
        } else {
            Err(String::from("Name cannot be empty"))
        }
    };

    let name_formatter: StringFormatter = &|input: &str| input.trim().to_owned();
    loop {
        let name = Text::new("Task name:")
            .with_help_message("Enter the name of the task")
            .with_validator(name_validator)
            .with_formatter(name_formatter)
            .prompt();

        match name {
            Ok(n) => return Ok(name_formatter(&n)),
            Err(InquireError::OperationCanceled) => {}
            Err(e) => return Err(Error::new(e)),
        }
    }
}
