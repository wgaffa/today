use std::collections::{hash_map::Entry, HashMap};

use termion::color;

use crate::Task;

#[derive(Debug, Clone, Copy)]
pub enum Visibility {
    Visible,
    Hidden,
}

#[derive(Debug, Clone, Copy)]
pub enum Size {
    Min(usize),
    Max(usize),
}

#[derive(Debug, Clone)]
pub struct Cell {
    content: String,
    visibility: Visibility,
    margin: Margin,
    size: Size,
}

impl Cell {
    pub fn new<T: Into<String>>(content: T) -> Self {
        Self {
            content: content.into(),
            visibility: Visibility::Visible,
            size: Size::Min(0),
            ..Default::default()
        }
    }

    pub fn visibility(&self) -> Visibility {
        self.visibility
    }

    pub fn with_visibility(self, visibility: Visibility) -> Self {
        Self { visibility, ..self }
    }

    pub fn with_margin<T: Into<Margin>>(self, margin: T) -> Self {
        Self {
            margin: margin.into(),
            ..self
        }
    }

    pub fn with_size(self, size: Size) -> Self {
        Self { size, ..self }
    }

    pub fn with_content<T: Into<String>>(self, content: T) -> Self {
        Self {
            content: content.into(),
            ..self
        }
    }

    pub fn content(&self) -> &str {
        &self.content
    }
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            content: String::new(),
            visibility: Visibility::Visible,
            margin: Default::default(),
            size: Size::Min(0),
        }
    }
}

impl std::fmt::Display for Cell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (out, width) = match (self.visibility, self.size) {
            (Visibility::Hidden, _) => return Ok(()),
            (Visibility::Visible, Size::Min(x)) => (self.content.as_str(), x),
            (Visibility::Visible, Size::Max(x)) => {
                (self.content.get(0..x).unwrap_or(self.content.as_str()), x)
            }
        };

        let left = self.margin.left;
        let right = self.margin.right;
        write!(f, "{:left$}{:width$}{:right$}", "", out, "")
    }
}

pub type Format = String;
pub trait TaskFormatter {
    fn format(&self, task: &Task) -> Format;
}

#[derive(Debug, Clone)]
pub struct TodayFormatter {
    columns: HashMap<Field, Column>,
}

impl TodayFormatter {
    pub fn new() -> Self {
        Self {
            columns: HashMap::new(),
        }
    }

    pub fn column(&mut self, field: Field) -> Entry<'_, Field, Column> {
        self.columns.entry(field)
    }

    pub fn insert<T: Into<Column>>(&mut self, field: Field, cell: T) {
        self.columns.insert(field, cell.into());
    }
}

impl Default for TodayFormatter {
    fn default() -> Self {
        Self::new()
    }
}

impl TaskFormatter for TodayFormatter {
    fn format(&self, task: &Task) -> Format {
        let id = task.id().as_ref().to_simple().to_string();
        let id = self
            .columns
            .get(&Field::Id)
            .cloned()
            .unwrap_or_default()
            .cell()
            .with_content(id);
        let name = Cell::new(task.name());
        let time = task.due().map_or(String::from("Now"), |x| {
            x.format("%Y-%m-%d %H:%M").to_string()
        });
        let time = Cell::new(time);
        format!(
            "{}{}{}{}: {}",
            id,
            color::Fg(color::LightRed),
            time,
            color::Fg(color::Reset),
            name
        )
    }
}

pub struct ListFormatter {
    columns: HashMap<Field, Column>,
}

impl ListFormatter {
    pub fn new() -> Self {
        Self {
            columns: HashMap::new(),
        }
    }

    pub fn column(&mut self, field: Field) -> Entry<'_, Field, Column> {
        self.columns.entry(field)
    }

    pub fn insert<T: Into<Column>>(&mut self, field: Field, cell: T) {
        self.columns.insert(field, cell.into());
    }
}

impl Default for ListFormatter {
    fn default() -> Self {
        Self::new()
    }
}

impl TaskFormatter for ListFormatter {
    fn format(&self, task: &Task) -> Format {
        let id = task.id().as_ref().to_simple().to_string();
        let id = self
            .columns
            .get(&Field::Id)
            .cloned()
            .unwrap_or_default()
            .cell()
            .with_content(id);
        let name = self
            .columns
            .get(&Field::Name)
            .cloned()
            .unwrap_or_default()
            .cell()
            .with_content(task.name());
        let time = task.due().map_or(String::from("Now"), |x| {
            x.format("%Y-%m-%d %H:%M").to_string()
        });
        let time = self
            .columns
            .get(&Field::Time)
            .cloned()
            .unwrap_or_default()
            .cell()
            .with_content(time);
        format!("{}{}{}", id, time, name)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Field {
    Id,
    Name,
    Time,
}

#[derive(Debug, Clone, Default)]
pub struct Column {
    cell: Cell,
}

impl Column {
    /// Create a new column using `cell` as the prototype.
    pub fn new(cell: Cell) -> Self {
        Self { cell }
    }

    /// Return a cloned copy of the cell. This is like a prototype to use for all
    /// cells that should be alike in the same column.
    pub fn cell(&self) -> Cell {
        self.cell.clone()
    }
}

impl From<Cell> for Column {
    fn from(cell: Cell) -> Self {
        Self { cell }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Margin {
    right: usize,
    left: usize,
}

impl Margin {
    pub fn new(left: usize, right: usize) -> Self {
        Self { left, right }
    }
}

impl From<(usize, usize)> for Margin {
    fn from(v: (usize, usize)) -> Self {
        let (left, right) = v;
        Self { left, right }
    }
}
