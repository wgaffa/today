use today::Task;
use chrono::prelude::*;

fn main() {
    let task = Task::new("Meet Dave").date(Utc::today());

    println!("{:#?}", task);
}
