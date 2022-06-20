use std::io::{stdout, Write};

use crossterm::{cursor, terminal, ExecutableCommand, QueueableCommand};

pub struct WatchMode {
    first_call: bool,
}

impl WatchMode {
    pub fn new() -> Self {
        stdout().execute(cursor::Hide).unwrap();
        Self { first_call: true }
    }
}

impl Drop for WatchMode {
    fn drop(&mut self) {
        stdout().execute(cursor::Show).unwrap();
    }
}

impl Write for WatchMode {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let mut stdout = stdout();
        self.first_call
            .then(|| self.first_call = false)
            .or_else(|| {
                stdout.queue(cursor::RestorePosition).unwrap();
                stdout
                    .queue(terminal::Clear(terminal::ClearType::FromCursorDown))
                    .unwrap();
                Some(())
            });

        stdout.queue(cursor::SavePosition).unwrap();
        let res = stdout.write(buf);

        res
    }

    fn flush(&mut self) -> std::io::Result<()> {
        stdout().flush()
    }
}
