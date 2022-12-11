// TODO: add comments
use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::error::Error;
use std::fmt::Display;
use std::fs;
use std::path::Path;
use std::str::FromStr;
use std::{convert::From, io};

use anyhow::ensure;
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use serde_with::{DeserializeFromStr, SerializeDisplay};

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
        Command::Add {
            text,
            priority,
            group,
        } => {
            t.add(text, priority, group)?;
        }
        Command::List => t.list(),
    }
    fs::write(
        format!("{HOME}/.rusty-todo.json"),
        serde_json::to_string_pretty(&t)?,
    )?;
    Ok(())
}

impl Tasks {
    // TODO: check for collisions?
    fn add(&mut self, todo: String, priority: u8, group: Option<String>) -> anyhow::Result<()> {
        ensure!(!todo.is_empty(), "task cannot be empty");
        ensure!(
            !todo.chars().all(char::is_whitespace),
            "task cannot be solely whitespace"
        );

        self.0.insert(
            Descriptor {
                priority,
                // Take 1 + the highest index
                id: self.0.keys().map(|d| d.id).max().unwrap_or(0) + 1,
                group,
            },
            todo,
        );

        self.reindex();

        Ok(())
    }

    fn list(&self) {
        // If on a loop iteration, the previous priority
        // was different, we emit a newline to make
        // sections
        let mut previous_priority: Option<u8> = None;

        for (
            Descriptor {
                priority,
                id,
                group,
            },
            todo,
        ) in self.0.iter().rev()
        {
            // Compare to previous priority (if it exists).
            // It will be None on the first loop iteration
            if let Some(inner) = previous_priority {
                if inner != *priority {
                    println!();
                }
            }
            previous_priority = Some(*priority);

            // stars indicate priority
            let stars = match (*priority).clamp(0, 3) {
                0 => " ".into(),
                x => format!(" ({})", "*".repeat(x.into())),
            };

            // Format the part of the output determining the group
            let group = if let Some(group) = group {
                format!(" ({group})")
            } else {
                "".into()
            };

            PRINTER
                .default(todo)
                .green(&format!(" ({id})"))
                .yellow(group)
                .bred(stars)
                .finish_nl();
        }
    }

    /// Reindex the tasks so that there are no holes in the indices and
    /// more important tasks have lower numbers
    fn reindex(&mut self) {
        self.0 = self
            .0
            .iter()
            .rev()
            .zip(1..)
            .map(|((desc, v), id)| {
                (
                    Descriptor {
                        priority: desc.priority,
                        id,
                        group: desc.group.clone(),
                    },
                    v.clone(),
                )
            })
            .collect::<BTreeMap<_, _>>();
    }

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

        // Figure out which indices need removing
        let remove = self
            .0
            .keys()
            .filter(|k| ids.contains(&k.id))
            .cloned()
            .collect::<Vec<_>>();

        // Deleting the tasks
        for desc @ Descriptor { id, .. } in remove {
            match self.0.remove(&desc) {
                Some(todo) => {
                    PRINTER
                        .default("Finished todo ")
                        .red(format!("({id})"))
                        .default(": ")
                        .default(todo)
                        .default(" :)")
                        .finish_nl();
                }
                None => println!("There was no task with index {RED}({id}){RESET}"),
            }
        }

        self.reindex();
    }
}

/// Get a line of user input with an optional prompt
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

#[derive(Parser, Debug)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
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
        group: Option<String>,
    },
    /// List all todos
    #[command(visible_alias = "l")]
    List,
}

struct App {
    tasks: Tasks,
    archive: Tasks,
    args: Cli,
}

#[derive(Debug, Serialize, Deserialize)]
struct Tasks(BTreeMap<Descriptor, String>);

#[derive(Eq, PartialEq, SerializeDisplay, DeserializeFromStr, Debug, Clone)]
struct Descriptor {
    priority: u8,
    id: usize,
    group: Option<String>,
}

impl Display for Descriptor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Format:
        // priotity-id-{0 for None, 1 for Some}group
        match &self.group {
            Some(group) => f.write_fmt(format_args!("{}-{}-1{}", self.priority, self.id, group)),
            None => f.write_fmt(format_args!("{}-{}-0", self.priority, self.id)),
        }
    }
}

impl FromStr for Descriptor {
    type Err = Box<dyn Error>;

    // Parsing out of the format:
    // priotity-id-{0 for None, 1 for Some}group
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut split = s.split('-').peekable();
        let priority = split.next().ok_or("no priority")?.parse::<u8>()?;
        let id = split.next().ok_or("no id")?.parse::<usize>()?;

        // If there are no more segments, there is no todo
        split.peek().ok_or("no todo provided")?;

        // Collect the rest of the splits into a string
        let rest = split.collect::<String>();

        // Option<String> was None if the rest starts with 0
        if rest.starts_with('0') {
            Ok(Descriptor {
                priority,
                id,
                group: None,
            })
        } else {
            Ok(Descriptor {
                priority,
                id,
                // Skip the 1 indicator
                group: Some(String::from(&rest[1..])),
            })
        }
    }
}

impl Ord for Descriptor {
    fn cmp(&self, other: &Self) -> Ordering {
        // First compare by priority
        match self.priority.cmp(&other.priority) {
            Ordering::Equal => {}
            ord => {
                return ord;
            }
        }

        // Then compare by group
        match self.group.cmp(&other.group) {
            Ordering::Equal => {}
            ord => {
                return ord;
            }
        }

        // Then compare by id
        match self.id.cmp(&other.id) {
            Ordering::Equal => {}
            ord => {
                return ord;
            }
        }

        Ordering::Equal
    }
}

impl PartialOrd for Descriptor {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
