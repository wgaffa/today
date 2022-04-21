use chrono::prelude::*;

use crate::{Task, TaskId, TaskName};

// Instruction ::= New | Edit
// Id ::= [0-9a-f]+
// Date ::= ([0-9]+ '-' [0-9]+ '-' [0-9]+)
// Time ::= [0-9]+ ':' [0-9]+
// Name ::= All
// DateTime ::= "Now" | Date Time
// Edit ::= Id ("edit" | "rm" | "remove") DateTime Name
// New ::= "new" Date Time? Name

#[derive(Debug, Clone)]
pub enum Program {
    Add(Task),
    Edit(Task),
    Remove(TaskId),
}

#[derive(Debug, Clone, Copy)]
pub enum ParseError {
    InvalidTime,
    ExpectedEOF,
    UnexpectedToken,
    UnexpectedEOF,
}

pub struct Parser<'a> {
    text: &'a str,
    position: usize,
}

impl<'a> Parser<'a> {
    pub fn new(text: &'a str) -> Self {
        Self { text, position: 0 }
    }

    fn skip_whitespace(&mut self) {
        let next_bounds = self.text.ceil_char_boundary(self.position);
        let next_pos = self.text[next_bounds..]
            .chars()
            .position(|x| !x.is_whitespace());
        self.position = next_pos.unwrap_or_else(|| self.text.len());
    }

    fn peek(&self) -> Option<char> {
        let next_char_index = self.text.ceil_char_boundary(self.position + 1);
        if next_char_index >= self.text.len() {
            return None;
        }

        self.text[next_char_index..].chars().next()
    }

    pub fn parse(&mut self) -> Result<Program, ParseError> {
        let instruction = self.instruction()?;

        if self.position <= self.text.len() {
            Err(ParseError::ExpectedEOF)
        } else {
            Ok(instruction)
        }
    }

    fn instruction(&mut self) -> Result<Program, ParseError> {
        self.skip_whitespace();
        let next_char = self.peek().ok_or(ParseError::UnexpectedEOF)?;

        if next_char == 'n' {
            self.add()
        } else {
            self.edit()
        }
    }

    fn add(&mut self) -> Result<Program, ParseError> {
        let action = self.text[self.position..]
            .chars()
            .position(|x| x.is_whitespace())
            .map_or_else(
                || &self.text[self.position..],
                |x| &self.text[self.position..x],
            );

        if action == "new" {
            self.position += 3;
            self.skip_whitespace();
            let datetime = self.datetime()?;

            self.skip_whitespace();
            let name = self.name()?;

            todo!()
        } else {
            return Err(ParseError::UnexpectedToken);
        }
    }

    fn edit(&mut self) -> Result<Program, ParseError> {
        todo!()
    }

    fn id(&mut self) -> Result<TaskId, ParseError> {
        todo!()
    }

    fn action(&mut self) -> Result<Program, ParseError> {
        todo!()
    }

    fn datetime(&mut self) -> Result<Option<DateTime<Utc>>, ParseError> {
        let next_char = self.peek().ok_or(ParseError::UnexpectedEOF)?;

        if next_char == 'N' {
            let keyword = self.text[self.position..]
                .chars()
                .position(|x| x.is_whitespace())
                .map_or_else(
                    || &self.text[self.position..],
                    |x| &self.text[self.position..x],
                );

            if keyword == "Now" {
                self.position += 3;
                Ok(None)
            } else {
                Err(ParseError::UnexpectedToken)
            }
        } else {
            let date = self.date()?;
            let time = self.time()?;

            Utc.from_utc_date(&date)
                .and_time(time)
                .map(|x| Some(x))
                .ok_or(ParseError::InvalidTime)
        }
    }

    fn date(&mut self) -> Result<NaiveDate, ParseError> {
        let is_dash = |x| x == '-';
        let index = self.text[self.position..]
            .chars()
            .position(is_dash)
            .ok_or(ParseError::UnexpectedEOF)?;
        let year = self.text[self.position..index]
            .parse::<i32>()
            .map_err(|_| ParseError::InvalidTime)?;
        self.position += index;

        let index = self.text[self.position..]
            .chars()
            .position(is_dash)
            .ok_or(ParseError::UnexpectedEOF)?;
        let month = self.text[self.position..index]
            .parse::<u32>()
            .map_err(|_| ParseError::InvalidTime)?;
        self.position += index;

        let index = self.text[self.position..]
            .chars()
            .position(|x| x.is_whitespace())
            .unwrap_or_else(|| self.text.len());
        let day = self.text[self.position..index]
            .parse::<u32>()
            .map_err(|_| ParseError::InvalidTime)?;
        self.position += index;

        NaiveDate::from_ymd_opt(year, month, day).ok_or(ParseError::InvalidTime)
    }

    fn time(&mut self) -> Result<NaiveTime, ParseError> {
        let hour = self.text[self.position..]
            .chars()
            .position(|x| x == ':')
            .ok_or(ParseError::UnexpectedEOF)
            .and_then(|x| {
                let pos = self.position;
                self.position += x + 1;
                self.text[pos..pos + x]
                    .parse::<u32>()
                    .map_err(|_| ParseError::InvalidTime)
            })?;

        let minute = self.text[self.position..]
            .chars()
            .position(|x| x.is_whitespace())
            .or_else(|| Some(self.text.len() - self.position))
            .ok_or(ParseError::UnexpectedEOF)
            .and_then(|x| {
                dbg!(x);
                let pos = self.position;
                self.position += x;
                self.text[pos..pos + x]
                    .parse::<u32>()
                    .map_err(|_| ParseError::InvalidTime)
            })?;

        NaiveTime::from_hms_opt(hour, minute, 0).ok_or(ParseError::InvalidTime)
    }

    fn name(&mut self) -> Result<TaskName, ParseError> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use test_case::test_case;

    #[test]
    fn time_should_parse_valid_time() {
        let time = "12:43";
        let mut parser = Parser::new(time);

        let result = parser.time();
        assert!(result.is_ok());
        assert_eq!(parser.position, 5);
        assert_eq!(result.unwrap(), NaiveTime::from_hms(12, 43, 0));
    }

    #[test_case("26:00")]
    #[test_case("3")]
    #[test_case(":33")]
    fn time_should_return_err_given_out_of_bounds_time(input: &str) {
        let mut parser = Parser::new(input);

        assert!(parser.time().is_err());
    }
}
