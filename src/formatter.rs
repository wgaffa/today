use termion::color;

use crate::Task;

#[derive(Debug, Clone, Copy)]
pub enum Visibility {
    Visible,
    Hidden,
}

#[derive(Debug, Clone)]
pub struct Cell {
    content: String,
    visibility: Visibility,
}

impl Cell {
    pub fn new<T: Into<String>>(content: T) -> Self {
        Self {
            content: content.into(),
            visibility: Visibility::Visible,
        }
    }

    pub fn visibility(&self) -> Visibility {
        self.visibility
    }

    pub fn with_visibility(self, visibility: Visibility) -> Self {
        Self { visibility, ..self }
    }

    pub fn content(&self) -> &str {
        &self.content
    }
}

impl std::fmt::Display for Cell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let out = match self.visibility {
            Visibility::Hidden => "",
            Visibility::Visible => &self.content,
        };

        write!(f, "{}", out)
    }
}

pub type Format = String;
pub trait TaskFormatter {
    fn format(&self, task: &Task) -> Format;
}

#[derive()]
pub struct SimpleFormatter;

impl TaskFormatter for SimpleFormatter {
    fn format(&self, task: &Task) -> Format {
        let id = task.id().as_ref().to_simple().to_string();
        let id = Cell::new(id)
            .with_visibility(Visibility::Hidden);
        let name = Cell::new(task.name());
        let time = task.due().map_or(String::from("Now"), |x| {
            x.format("%Y-%m-%d %H:%M").to_string()
        });
        let time = Cell::new(time);
        format!(
            "{}{}{:>16}{}: {}",
            id,
            color::Fg(color::LightRed),
            time,
            color::Fg(color::Reset),
            name
        )
    }
}
