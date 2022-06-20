use std::io::{stdout, Write};

use crossterm::{cursor, style, terminal, ExecutableCommand};

pub trait OutputMode {
    fn write(&mut self, buf: &str) -> std::io::Result<()>;
}

pub struct WatchMode {
    pos: (u16, u16),
}

impl WatchMode {
    pub fn new() -> Self {
        stdout().execute(cursor::Hide).unwrap();
        Self {
            pos: cursor::position().expect("Could not read cursor position"),
        }
    }
}

impl Drop for WatchMode {
    fn drop(&mut self) {
        stdout().execute(cursor::Show).unwrap();
    }
}

impl OutputMode for WatchMode {
    fn write(&mut self, buf: &str) -> std::io::Result<()> {
        let mut stdout = stdout();

        let lines_count = buf.lines().count() as u16;
        let (_, row_size) = terminal::size().expect("Could not get terminal size");

        // clamp to the bottom of the screen
        if self.pos.1 + lines_count > row_size {
            self.pos.1 = row_size.saturating_sub(lines_count).saturating_sub(1);
        }

        crossterm::queue! {
            stdout,
            cursor::MoveTo(self.pos.0, self.pos.1),
            terminal::Clear(terminal::ClearType::FromCursorDown),
            style::Print(buf),
        }?;

        stdout.flush()
    }
}

impl OutputMode for std::io::Stdout {
    fn write(&mut self, buf: &str) -> std::io::Result<()> {
        write!(self, "{}", buf)
    }
}
