use crate::Task;

pub trait Repository {
    type Err;

    fn all(&mut self) -> Result<&[Task], Self::Err>;
}
