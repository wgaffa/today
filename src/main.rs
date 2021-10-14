use std::error::Error;

mod ui;

fn main() -> Result<(), Box<dyn Error>>{
    let _ = ui::menu();

    let task = ui::prompt_task()?;
    println!("{:?}", task);

    Ok(())
}
