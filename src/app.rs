use crate::AppPaths;
use today::{partial_config::Run, repository::Repository};

pub struct App {
    config: AppPaths<Run>,
    repo: Box<dyn Repository<Err = std::io::Error>>,
}

impl App {
    pub fn new<R: Repository<Err = std::io::Error> + 'static>(config: AppPaths<Run>, repo: R) -> Self {
        Self {
            config,
            repo: Box::new(repo),
        }
    }

    pub fn run(&mut self) -> anyhow::Result<()> {
        todo!()
    }
}
