use std::io;

use chrono::prelude::*;
use clap::ArgMatches;

use today::parser::program::{Parser, Program};

use super::Command;

pub fn parse(command: &str, mut matches: ArgMatches) -> Option<Command> {
    match command {
        "add" => Some(add(matches)),
        "list" => Some(Command::List),
        "remove" => {
            let id = matches.remove_one::<String>("id").unwrap();
            Some(Command::Remove(id))
        }
        "today" => Some(Command::Today {
            watch_mode: matches.remove_one("watch").unwrap_or(false),
        }),
        "edit" => Some(Command::Edit { program: edit() }),
        _ => None,
    }
}

fn edit() -> Vec<Program> {
    io::stdin()
        .lines()
        .map(|line| {
            let line = line.unwrap();
            let mut parser = Parser::new(&line);
            parser.parse()
        })
        .inspect(|result| {
            if let Err(e) = result {
                eprintln!("Failed to parse: {e}");
            }
        })
        .flatten()
        .collect()
}

fn add(mut matches: ArgMatches) -> Command {
    let name = matches.remove_one("name");
    let now = matches.contains_id("now").then_some(None);
    let due = matches.remove_one::<DateTime<Utc>>("due").map(|x| Some(x));
    let due = now.or(due);
    Command::Add { name, due }
}
