use std::{fs, io::ErrorKind, path::PathBuf};

use crate::{Task, TaskList};

pub struct JsonRepository {
    path: PathBuf,
    tasks: TaskList,
}

impl JsonRepository {
    pub fn new<P: Into<PathBuf>>(path: P) -> Self {
        Self {
            path: path.into(),
            tasks: TaskList::new(),
        }
    }
}

impl crate::repository::Repository for JsonRepository {
    type Err = std::io::Error;

    fn all(&mut self) -> Result<&[Task], Self::Err> {
        let file_content = match fs::read_to_string(&self.path) {
            Err(err) if err.kind() == ErrorKind::NotFound => Ok(String::from("[]")), // Empty JSON array
            x => x,
        }?;

        let db = serde_json::from_str::<Vec<Task>>(&file_content)?;
        self.tasks.add_range(&db);

        Ok(self.tasks.as_slice())
    }
}
