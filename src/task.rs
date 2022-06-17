use std::convert::AsRef;

use chrono::prelude::*;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

/// `TaskName` is a any non empty string with at least one printable character with surrounding
/// whitespaces trimmed. `TaskName` is compared case insensitive.
/// ```
/// use today::task::TaskName;
///
/// let name = TaskName::new("  Leading and trailing whitespaces\t").unwrap();
///
/// assert_eq!(name.as_str(), "Leading and trailing whitespaces");
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(transparent)]
#[repr(transparent)]
pub struct TaskName(String);

impl std::fmt::Display for TaskName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TaskName {
    pub fn new(value: &str) -> Option<Self> {
        let value = value.trim();
        if value.is_empty() {
            None
        } else {
            Some(Self(value.to_owned()))
        }
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    pub fn into_inner(self) -> String {
        self.0
    }
}

impl PartialEq<&str> for TaskName {
    fn eq(&self, other: &&str) -> bool {
        self.0 == *other
    }
}

impl AsRef<str> for TaskName {
    fn as_ref(&self) -> &str {
        self.0.as_str()
    }
}

impl AsRef<String> for TaskName {
    fn as_ref(&self) -> &String {
        &self.0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(transparent)]
pub struct TaskId(Uuid);

impl std::fmt::Display for TaskId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.to_simple_ref())
    }
}

impl TaskId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl std::convert::AsRef<Uuid> for TaskId {
    fn as_ref(&self) -> &Uuid {
        &self.0
    }
}

impl Default for TaskId {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct Task {
    #[serde(default)]
    id: TaskId,
    name: TaskName,
    due: Option<DateTime<Utc>>,
}

impl Task {
    /// Create a new task with given name
    /// ```
    /// use today::{Task, TaskName};
    ///
    /// let task = Task::new(TaskName::new("Meet Dave").unwrap());
    ///
    /// assert_eq!(task.name(), "Meet Dave");
    /// ```
    pub fn new(name: TaskName) -> Self {
        Self {
            id: Default::default(),
            name,
            due: None,
        }
    }

    pub fn with_name(mut self, name: TaskName) -> Self {
        self.name = name;
        self
    }

    pub fn with_due(mut self, due: Option<DateTime<Utc>>) -> Self {
        self.due = due;
        self
    }

    /// Add a due date in UTC for a task
    /// ```
    /// use today::{Task, TaskName};
    /// use chrono::prelude::*;
    ///
    /// let task = Task::new(TaskName::new("Meet Dave").unwrap())
    ///     .with_date(Utc.ymd(2020, 2, 23));
    ///
    /// assert_eq!(task.due(), Some(&Utc.ymd(2020, 2, 23).and_hms(0, 0, 0)));
    /// ```
    pub fn with_date(mut self, due: Date<Utc>) -> Self {
        self.due = Some(due.and_hms(0, 0, 0));
        self
    }

    /// Add a due date and time in UTC for a task
    /// ```
    /// use today::{Task, TaskName};
    /// use chrono::prelude::*;
    ///
    /// let task = Task::new(TaskName::new("Meet Dave").unwrap())
    ///     .with_date_time(Utc.ymd(2020, 2, 23)
    ///     .and_hms(15, 30, 00));
    ///
    /// assert_eq!(task.due(), Some(&Utc.ymd(2020, 2, 23).and_hms(15, 30, 00)));
    /// ```
    pub fn with_date_time(mut self, due: DateTime<Utc>) -> Self {
        self.due = Some(due);
        self
    }

    /// Add a time to the task if date has been set first.
    /// If `due` is None then this has no effect
    pub fn and_time(mut self, time: NaiveTime) -> Self {
        self.due = self.due.and_then(|x| x.date().and_time(time));
        self
    }

    /// Get the name of the task
    pub fn name(&self) -> &str {
        self.name.0.as_str()
    }

    /// Get the date of the task.
    ///
    /// None represents a task to be done as soon as possible
    pub fn due(&self) -> Option<&DateTime<Utc>> {
        self.due.as_ref()
    }

    /// Get the id of the task.
    pub fn id(&self) -> &TaskId {
        &self.id
    }
}

impl std::fmt::Display for Task {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = &self.name;
        let date = self.due.map_or(String::from("ASAP"), |x| {
            x.format("%Y-%m-%d %H:%M").to_string()
        });
        write!(f, "{name} {date}")
    }
}

#[derive(Debug, Clone, Error)]
pub enum TaskError {
    #[error("Invalid id '{0}'")]
    InvalidId(TaskId),
}

#[derive(Debug)]
pub struct TaskList {
    tasks: Vec<Task>,
}

impl TaskList {
    pub fn new() -> Self {
        Self { tasks: Vec::new() }
    }

    pub fn add(&mut self, task: Task) {
        self.tasks.push(task);
    }

    pub fn add_range(&mut self, tasks: &[Task]) {
        self.tasks.extend_from_slice(tasks);
    }

    /// Remove a task from the list
    pub fn remove(&mut self, task_id: &TaskId) {
        self.tasks.retain(|x| x.id != *task_id);
    }

    pub fn edit(&mut self, task: Task) -> Result<Task, TaskError> {
        let filtered_tasks = self
            .tasks
            .iter()
            .position(|x| x.id() == task.id());

        match filtered_tasks {
            None => Err(TaskError::InvalidId(task.id().clone())),
            Some(index) => {
                let old = std::mem::replace(&mut self.tasks[index], task);
                Ok(old)
            }
        }
    }

    /// Returns an iterator over all tasks that are due today
    pub fn today(&self) -> Today<'_> {
        Today::new(&self.tasks)
    }

    pub fn iter(&self) -> std::slice::Iter<Task> {
        self.tasks.iter()
    }

    pub fn as_slice(&self) -> &[Task] {
        &self.tasks
    }
}

impl Default for TaskList {
    fn default() -> Self {
        Self::new()
    }
}

impl std::iter::Extend<Task> for TaskList {
    fn extend<T: IntoIterator<Item = Task>>(&mut self, iter: T) {
        self.tasks.extend(iter.into_iter().sorted().dedup());
    }
}

pub struct Today<'a> {
    slice: &'a [Task],
    today: Date<Utc>,
}

impl<'a> Iterator for Today<'a> {
    type Item = &'a Task;

    fn next(&mut self) -> Option<Self::Item> {
        for i in 0..self.slice.len() {
            let is_due = self.slice[i].due().map_or(true, |x| self.today >= x.date());
            if is_due {
                let task = &self.slice[i];
                self.slice = &self.slice[i + 1..];
                return Some(task);
            }
        }

        None
    }
}

impl<'a> Today<'a> {
    pub fn new(slice: &'a [Task]) -> Self {
        Self {
            slice,
            today: Utc::today(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case("" => None)]
    #[test_case("  leading" => Some(TaskName("leading".to_string())))]
    #[test_case("trailing "  => Some(TaskName("trailing".to_string())))]
    #[test_case("   mixed\n "  => Some(TaskName("mixed".to_string())))]
    fn new_taskname(input: &str) -> Option<TaskName> {
        TaskName::new(input)
    }
}
