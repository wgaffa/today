use std::error::Error;
use today::Task;
use chrono::prelude::*;
use inquire::{CustomType, DateSelect, Text};
use inquire::formatter::StringFormatter;
use inquire::ui::{RenderConfig, Styled};
use inquire::validator::StringValidator;
use inquire::error::{InquireError, InquireResult};

fn main() -> Result<(), Box<dyn Error>>{
    let task = prompt_task()?;
    println!("{:?}", task);

    Ok(())
}

fn prompt_task() -> Result<Task, Box<dyn Error>> {
    let name = prompt_name()?;

    let due = prompt_due()?;

    let task = Task::new(name);
    let task = if let Some(date) = due {
        let time = prompt_time()?;
        if let Some(local_time) = time {
            task.with_date_time(Utc.from_local_datetime(&date.and_time(local_time)).unwrap())
        } else {
            task.with_date(Utc.from_local_date(&date).unwrap())
        }
    } else {
        task
    };

    Ok(task)
}

fn prompt_time() -> InquireResult<Option<NaiveTime>> {
    let time_style = RenderConfig::default_colored()
        .with_canceled_prompt_indicator(Styled::new("00:00:00"));
    CustomType::<NaiveTime>::new("Time:")
        .with_default((NaiveTime::from_hms(0, 0, 0), &|x| format!("{}", x)))
        .with_placeholder("HH:MM:SS")
        .with_parser(&|x| {
            match NaiveTime::parse_from_str(x, "%H:%M") {
                Ok(time) => Ok(time),
                Err(_) => Err(()),
            }
        })
        .with_render_config(time_style)
        .prompt_skippable()
}

fn prompt_due() -> InquireResult<Option<NaiveDate>> {
    let date_style = RenderConfig::default_colored()
        .with_canceled_prompt_indicator(Styled::new("As soon as possible"));

    DateSelect::new("Due date:")
        .with_help_message("Press ESC to set task to be due as soon as possible")
        .with_min_date(Utc::today().naive_utc())
        .with_vim_mode(true)
        .with_render_config(date_style)
        .prompt_skippable()
}

fn prompt_name() -> Result<String, Box<dyn Error>> {
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
            Err(InquireError::OperationCanceled) => {},
            Err(e) => return Err(Box::new(e)),
        }
    };
}
