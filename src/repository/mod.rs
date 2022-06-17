use crate::TaskList;

pub trait Repository {
    type Err;

    fn all(&self) -> Result<TaskList, Self::Err>;

    fn save(&self, tasks: TaskList) -> Result<(), Self::Err>;
}
