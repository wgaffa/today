use std::{fs, io::ErrorKind, path::PathBuf};

use serde_json;

use crate::{Task, TaskList};

pub struct JsonRepository {
    path: PathBuf,
}

impl JsonRepository {
    pub fn new<P: Into<PathBuf>>(path: P) -> Self {
        Self { path: path.into() }
    }
}

impl crate::repository::Repository for JsonRepository {
    type Err = std::io::Error;

    fn all(&self) -> Result<TaskList, Self::Err> {
        let file_content = match fs::read_to_string(&self.path) {
            Err(err) if err.kind() == ErrorKind::NotFound => Ok(String::from("[]")), // Empty JSON array
            x => x,
        }?;

        let db = serde_json::from_str::<Vec<Task>>(&file_content)?;
        Ok(TaskList::from(db))
    }

    fn save(&self, tasks: TaskList) -> Result<(), Self::Err> {
        let json = serde_json::to_string(&tasks.as_slice())?;
        let directory = self.path
            .parent()
            .expect("Expected a directory for the file");

        if let Err(err) = fs::metadata(directory) {
            if err.kind() == ErrorKind::NotFound {
                fs::create_dir_all(directory)?
            } else {
                return Err(err);
            }
        }

        fs::write(&self.path, &json)
    }
}
