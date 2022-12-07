use std::cmp::Ordering;
// TODO: priority
use std::collections::{HashMap, BTreeSet, BTreeMap};
use std::fmt::format;
use std::{fs, cmp};
use std::path::Path;
use std::{convert::From, io};

use anyhow::ensure;
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};

use todo::color::{Printer, RED, RESET};

const PRINTER: Printer = Printer();
const HOME: &str = env!("HOME");

/// Make sure the todos file exists, otherwise create it
fn file_setup() -> anyhow::Result<()> {
    match Path::try_exists(Path::new(&format!("{HOME}/.rusty-todo.json"))) {
        Ok(true) => return Ok(()),
        // Broken symlinks or errors; see docs
        Ok(false) | Err(_) => {
            fs::File::create(Path::new(&format!("{HOME}/.rusty-todo.json")))?;
        }
    }
    Ok(())
}

fn main() -> anyhow::Result<()> {
    file_setup()?;
    let args = Cli::parse();
    let tasks = fs::read_to_string(format!("{HOME}/.rusty-todo.json"))?;
    let mut t: Tasks = serde_json::from_str(&tasks)?;
    match args.command {
        Command::Delete { id } => t.delete(id),
        Command::Add { text, priority, group } => t.add(text, priority, group)?,
        Command::List => t.list(),
    }
    fs::write(
        format!("{HOME}/.rusty-todo.json"),
        serde_json::to_string(&t)?,
    )?;
    Ok(())
}

impl Tasks {
    // TODO: check for collisions
    fn add(&mut self, text: String, priority: u8, group: Option<String>) -> anyhow::Result<()> {
        ensure!(!text.is_empty(), "task cannot be empty");
        ensure!(
            !text.chars().all(char::is_whitespace),
            "task cannot be solely whitespace"
        );
        self.0.insert(
            self.0.keys().max().unwrap_or(&0) + 1,
            Todo {
                text,
                priority,
                group,
            },
        );
        Ok(())
    }

    fn list(&self) {
        for (index, Todo { text, priority, group }) in self.0.iter() {
            let stars = match (*priority).clamp(0, 3) {
                0 => " ".into(),
                x => format!(" ({})", "*".repeat(x.into()))

            };

            let group = if let Some(group) = group {
                format!(" ({group})")
            } else {
                "".into()
            };

            PRINTER
                .default(text)
                .green(&format!(" ({index})"))
                .yellow(group)
                .bred(stars)
                .finish_nl();
        }
    }
    
    fn reindex(&self) {}

    fn delete(&mut self, id: Option<usize>) {
        // Get list of ids or provided id
        let ids = match id {
            Some(id) => vec![id],
            None => {
                // TODO: improve ux
                println!("No Todo selected: which one(s) would you like to delete?");
                self.list();
                let tasks = get_user_input(None);
                let (ids, errs): (Vec<_>, Vec<_>) = tasks
                    .split(',')
                    .map(str::trim)
                    .map(str::parse::<usize>)
                    .partition(Result::is_ok);
                for err in errs {
                    println!("{}", err.unwrap_err());
                }
                ids.into_iter().map(Result::unwrap).collect()
            }
        };
        for id in ids {
            match self.0.remove(&id) {
                Some(todo) => {
                    PRINTER
                        .default("Finished todo ")
                        .red(format!("({id})"))
                        .default(": ")
                        .default(todo.text)
                        .default(" :)")
                        .finish_nl();
                }
                None => println!("There was no task with index {RED}({id}){RESET}"),
            }
        }
    }
}

fn get_user_input(prompt: Option<&str>) -> String {
    if let Some(str) = prompt {
        println!("{str}");
    }
    let mut resp = String::new();
    io::stdin()
        .read_line(&mut resp)
        .expect("faile to read line");
    resp.trim().into()
}

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Delete a todo
    #[command(visible_aliases=["d", "-"])]
    Delete {
        /// The index of the todo to delete
        id: Option<usize>,
    },
    /// Add a todo
    #[command(visible_aliases=["a", "+"])]
    Add {
        /// The text of the todo to be added
        text: String,
        /// The priority of the task. Pass
        /// this flag once for low priority, twice
        /// for medium priority, and 3+ times for high
        /// priority.
        #[arg(short, long, action = clap::ArgAction::Count)]
        priority: u8,
        #[arg(short, long)]
        group: Option<String>
    },
    /// List all todos
    #[command(visible_alias = "l")]
    List,
}


#[derive(Debug, Serialize, Deserialize)]
struct Tasks(BTreeMap<usize, Todo>);

#[derive(Parser, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
struct Todo {
    text: String,
    priority: u8,
    group: Option<String>,
}

#[derive(Eq, PartialEq)]
struct Descriptor {
    priority: u8,
    index: usize,
    group: Option<String>
}

impl Ord for Descriptor {
    fn cmp(&self, other: &Self) -> Ordering{
        // First compare by priority
        match self.priority.cmp(&other.priority) {
            Ordering::Equal => {},
            ord => { return ord; }
        }

        // Then compare by group
        match self.group.cmp(&other.group) {
            Ordering::Equal => {},
            ord => { return ord; }
        }

        // Nothing else matters
        Ordering::Equal
    }
}

impl PartialOrd for Descriptor {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}