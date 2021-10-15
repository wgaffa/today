use chrono::prelude::*;

#[derive(Debug)]
pub struct Task {
    name: String,
    due: Option<DateTime<Utc>>,
}

impl Task {
    /// Create a new task with given name
    /// ```
    /// use today::Task;
    ///
    /// let task = Task::new("Meet Dave");
    ///
    /// assert_eq!(task.name(), "Meet Dave");
    /// ```
    pub fn new<T: Into<String>>(name: T) -> Self {
        Self {
            name: name.into(),
            due: None,
        }
    }

    /// Add a due date in UTC for a task
    /// ```
    /// use today::Task;
    /// use chrono::prelude::*;
    ///
    /// let task = Task::new("Meet Dave").with_date(Utc.ymd(2020, 2, 23));
    ///
    /// assert_eq!(task.due(), Some(&Utc.ymd(2020, 2, 23).and_hms(0, 0, 0)));
    /// ```
    pub fn with_date(mut self, due: Date<Utc>) -> Self {
        self.due = Some(due.and_hms(0, 0, 0));
        self
    }

    /// Add a due date and time in UTC for a task
    /// ```
    /// use today::Task;
    /// use chrono::prelude::*;
    ///
    /// let task = Task::new("Meet Dave").with_date_time(Utc.ymd(2020, 2, 23).and_hms(15, 30, 00));
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
        self.due = self.due.map(|x| x.date().and_time(time)).flatten();
        self
    }

    /// Get the name of the task
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the date of the task.
    ///
    /// None represents a task to be done as soon as possible
    pub fn due(&self) -> Option<&DateTime<Utc>> {
        self.due.as_ref()
    }
}

#[derive(Debug)]
pub struct TaskManager {
    tasks: Vec<Task>,
}

impl TaskManager {
    pub fn new() -> Self {
        Self { tasks: Vec::new() }
    }

    pub fn add(&mut self, task: Task) {
        self.tasks.push(task);
    }
}
