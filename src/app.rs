use crate::AppPaths;
use today::partial_config::Run;

pub struct App {
    config: AppPaths<Run>,
}

impl App {
    pub fn new(config: AppPaths<Run>) -> Self {
        Self { config }
    }

    pub fn run(&mut self) -> anyhow::Result<()> {
        todo!()
    }
}
