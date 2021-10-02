use chrono::prelude::*;

#[derive(Debug)]
pub struct Task {
    name: String,
    due: Option<DateTime<Utc>>,
}

impl Task {
    pub fn new<T: Into<String>>(name: T) -> Self {
        Self {
            name: name.into(),
            due: None,
        }
    }

    pub fn date(self, due: Date<Utc>) -> Self {
        Self {
            due: Some(due.and_hms(0, 0, 0)),
            .. self
        }
    }

    pub fn date_time(self, due: DateTime<Utc>) -> Self {
        Self {
            due: Some(due),
            .. self
        }
    }
}
