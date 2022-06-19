use std::{
    io::{stdout, Write},
    sync::mpsc::Receiver,
};

use chrono::{prelude::*, TimeZone};
use crossterm::{cursor, terminal, ExecutableCommand, QueueableCommand};

use today::{
    formatter::{self, Cell, Field, ListFormatter, TodayFormatter, Visibility},
    parser::program::Program,
    partial_config::Run,
    repository::Repository,
    task::{TaskList, TaskName},
};

use crate::{
    cli,
    commands::{self, Command},
    ui,
    AppPaths,
};

pub struct App {
    config: AppPaths<Run>,
    repo: Box<dyn Repository<Err = std::io::Error>>,
    file_changed: Option<Receiver<()>>,
}

impl App {
    pub fn new<R>(config: AppPaths<Run>, repo: R) -> Self
    where
        R: Repository<Err = std::io::Error> + 'static,
    {
        Self {
            config,
            repo: Box::new(repo),
            file_changed: None,
        }
    }

    pub fn run(&mut self) -> anyhow::Result<()> {
        match self.config.command.take() {
            Command::List => self.list(),
            Command::Today { .. } => self.today(),
            Command::Remove(x) => self.remove(&x),
            Command::Add { name, due } => self.add(name, due),
            Command::Edit { program } => self.edit(program),
            _ => self.interactive(),
        }
    }

    pub fn with_event_file_changed(self, reciever: Receiver<()>) -> Self {
        Self {
            file_changed: Some(reciever),
            ..self
        }
    }

    fn add(&self, name: Option<String>, due: Option<Option<DateTime<Utc>>>) -> anyhow::Result<()> {
        let name = name
            .and_then(|x| TaskName::new(&x))
            .or_else(|| TaskName::new(&ui::prompt_name().ok()?))
            .expect("Could not parse or get a correct taskname from user");

        let due: Option<DateTime<Utc>> = due.unwrap_or_else(|| {
            ui::prompt_due().ok().and_then(|d| {
                d.and_then(|d| {
                    ui::prompt_time()
                        .ok()
                        .map(|t| Utc.from_local_datetime(&d.and_time(t)).unwrap())
                })
            })
        });

        let mut tasks = TaskList::from(self.repo.all()?);

        cli::add(name, due, &mut tasks)?;

        Ok(self.repo.save(tasks)?)
    }

    fn list(&self) -> anyhow::Result<()> {
        let tasks = TaskList::from(self.repo.all()?);
        let shortest_id = commands::shortest_id_length(&tasks).max(5);
        let mut formatter = ListFormatter::new();

        let default_cell = Cell::default().with_margin((0, 1));
        formatter.insert(
            Field::Id,
            default_cell
                .clone()
                .with_size(formatter::Size::Max(shortest_id)),
        );
        formatter.insert(Field::Name, default_cell.clone().with_margin((0, 0)));
        formatter.insert(Field::Time, default_cell);

        println!("{}", commands::list(&tasks, &formatter));

        Ok(())
    }

    fn today(&self) -> anyhow::Result<()> {
        if let Some(ref rx) = self.file_changed {
            let mut stdout = stdout();

            stdout.execute(cursor::Hide).unwrap();
            loop {
                stdout.queue(cursor::RestorePosition).unwrap();
                stdout
                    .queue(terminal::Clear(terminal::ClearType::FromCursorDown))
                    .unwrap();

                stdout.queue(cursor::SavePosition).unwrap();
                let output = self.today_impl()?;
                stdout.write_all(output.as_bytes()).unwrap();

                stdout.queue(cursor::RestorePosition).unwrap();
                stdout.flush().unwrap();

                if let Err(_) = rx.recv() {
                    break;
                }
            }

            stdout.execute(cursor::Show).unwrap();
        } else {
            println!("{}", self.today_impl()?);
        }

        Ok(())
    }

    fn today_impl(&self) -> anyhow::Result<String> {
        let mut formatter = TodayFormatter::new();
        formatter.insert(
            Field::Id,
            Cell::default().with_visibility(Visibility::Hidden),
        );

        let tasks = TaskList::from(self.repo.all()?);
        Ok(commands::list(tasks.today(), &formatter))
    }

    fn remove(&self, id: &str) -> anyhow::Result<()> {
        let mut tasks = TaskList::from(self.repo.all()?);
        cli::remove(id, &mut tasks)?;

        Ok(self.repo.save(tasks)?)
    }

    fn edit(&self, programs: Vec<Program>) -> anyhow::Result<()> {
        let mut tasks = TaskList::from(self.repo.all()?);

        for program in programs {
            match program {
                Program::Edit { id, name, due } => {
                    let filtered_tasks = tasks
                        .iter()
                        .filter(|x| x.id().to_string().starts_with(&id))
                        .collect::<Vec<_>>();

                    match filtered_tasks.len() {
                        0 => eprintln!("No task found with the id '{}'", id),
                        1 => {
                            let new_task = filtered_tasks[0].clone().with_name(name).with_due(due);
                            if let Err(e) = tasks.edit(new_task) {
                                eprintln!("Unable to edit the task: {e}");
                            }
                        }
                        _ => {
                            eprintln!("More than one possible task was found with the id '{}'", id)
                        }
                    }
                }
                Program::Add(task) => tasks.add(task),
                Program::Remove(partial_id) => cli::remove(&partial_id, &mut tasks)?,
                _ => {}
            }
        }

        Ok(self.repo.save(tasks)?)
    }

    fn interactive(&self) -> anyhow::Result<()> {
        let mut tasks = TaskList::from(self.repo.all()?);
        let mut formatter = TodayFormatter::new();
        formatter.insert(
            Field::Id,
            Cell::default().with_visibility(Visibility::Hidden),
        );
        formatter.insert(Field::Name, Cell::default().with_margin((0, 1)));

        let mut formatter = ListFormatter::new();

        let default_cell = Cell::default().with_margin((0, 1));
        formatter.insert(
            Field::Id,
            default_cell.clone().with_visibility(Visibility::Hidden),
        );
        formatter.insert(Field::Name, default_cell.clone());
        formatter.insert(Field::Time, default_cell.clone());

        loop {
            let option = ui::menu()?;

            match option {
                ui::MenuOption::Quit => break,
                ui::MenuOption::Add => commands::add(ui::prompt_task, &mut tasks)?,
                ui::MenuOption::List => {
                    let max_name_length = tasks
                        .iter()
                        .map(|x| x.name().len())
                        .max()
                        .unwrap_or_default();

                    let col = formatter
                        .column(Field::Name)
                        .or_insert_with(|| default_cell.clone().into());
                    *col = Cell::default()
                        .with_size(formatter::Size::Max(max_name_length))
                        .into();

                    println!("{}", commands::list(tasks.as_slice(), &formatter))
                }
                ui::MenuOption::Remove => commands::remove(ui::prompt_task_remove, &mut tasks)?,
                ui::MenuOption::Today => println!("{}", commands::list(tasks.today(), &formatter)),
            }
        }

        Ok(self.repo.save(tasks)?)
    }
}
