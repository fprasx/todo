use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::fmt::Display;
use std::fs;
use std::path::Path;
use std::{convert::From, io};

use anyhow::ensure;
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

use todo::color::{Printer, RED, RESET};

const PRINTER: Printer = Printer::new();
const HOME: &str = env!("HOME");

/// Make sure the todos file exists, otherwise create it
fn file_setup() -> anyhow::Result<()> {
    match Path::try_exists(Path::new(&format!("{HOME}/.rusty-todo.json"))) {
        Ok(true) => { return Ok(()) },
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

    // Retrieve tasks
    let tasks = fs::read_to_string(format!("{HOME}/.rusty-todo.json"))?;
    let mut t: Todos = serde_json::from_str(&tasks)?;

    // Do what the user asked
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
        Command::Edit {
            id,
            priority,
            group,
        } => {
            t.edit(id, priority, group);
        }
    }

    // Write updates back to file
    fs::write(
        format!("{HOME}/.rusty-todo.json"),
        serde_json::to_string_pretty(&t)?,
    )?;

    Ok(())
}

impl Todos {
    fn edit(&mut self, id: usize, priority: u8, group: Option<String>) {
        let bundle = self.get(id);
        if let Some((desc, text)) = bundle {
            let new_desc = Descriptor {
                priority: priority as usize,
                id,
                group,
            };
            self.0.remove(&desc);
            self.0.insert(new_desc, text);
            self.reindex();
        } else {
            PRINTER
                .default("Task ")
                .red(format!("({id})"))
                .default(" does not exist")
                .print();
        }
    }

    fn get(&self, id: usize) -> Option<(Descriptor, String)> {
        self.0
            .iter()
            .find(|(desc, _)| desc.id == id as usize)
            .map(|(desc, text)| (desc.clone(), text.clone()))
    }

    // TODO: check for collisions?
    fn add(&mut self, todo: String, priority: u8, group: Option<String>) -> anyhow::Result<()> {
        ensure!(!todo.is_empty(), "task cannot be empty");
        ensure!(
            !todo.chars().all(char::is_whitespace),
            "task cannot be solely whitespace"
        );

        self.0.insert(
            Descriptor {
                priority: priority as usize,
                // Take 1 + the highest index
                id: self.0.keys().map(|d| d.id).max().unwrap_or(0) + 1,
                group,
            },
            todo,
        );

        self.reindex();

        Ok(())
    }

    fn list(&mut self) {
        // If on a loop iteration, the previous priority or group
        // was different, we emit a newline to make sections.
        // This works nicely because of the way Ord is
        // implemented for Descriptor and because BTreeMap::iter
        // returns items in sorted order
        let mut previous_priority: Option<usize> = None;
        let mut previous_group: &Option<String> = &None;

        if self.0.is_empty() {
            PRINTER.purple("Nothing!").newline().print();
            return;
        }

        for (
            desc @ Descriptor {
                priority, group, ..
            },
            todo,
        ) in self.0.iter().rev()
        {
            // If priority or group changes, print a newline
            if let Some(inner) = previous_priority {
                if inner != *priority || previous_group != group {
                    println!()
                }
            }
            previous_priority = Some(*priority);
            previous_group = group;

            PRINTER
                .default(todo)
                .default(desc.to_string())
                .newline()
                .print();
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
                // Prompt user for input
                println!("No Todo selected: which one(s) would you like to delete?");
                self.list();
                PRINTER.green("> ").print();

                let tasks = get_user_input(None);

                // Parse id's separated by commas
                let (ids, errs): (Vec<_>, Vec<_>) = tasks
                    .split(',')
                    .map(str::trim)
                    .map(str::parse::<usize>)
                    .partition(Result::is_ok);

                // Print out any errors
                for err in errs {
                    println!("Error parsing todo indices: {}", err.unwrap_err());
                }

                // Return all Ok's
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
                    // Confirm the successful deletion
                    PRINTER
                        .default("Finished todo ")
                        .green(format!("({id})"))
                        .default(": ")
                        .default(todo)
                        .purple(" :)")
                        .newline()
                        .print();
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
        priority: u8, // must be u8, otherwise clap erros
        #[arg(short, long)]
        group: Option<String>,
    },
    /// List all todos
    #[command(visible_alias = "l")]
    List,
    /// Edit the priority and group of a command using the same syntax as
    /// adding a command
    #[command(visible_alias = "e")]
    Edit {
        id: usize,
        #[arg(short, long, action = clap::ArgAction::Count)]
        priority: u8, // must be u8, otherwise clap erros
        #[arg(short, long)]
        group: Option<String>,
    },
}

// JSON keys cannot be structs so we use serde_as
// to serialize the BTreeMap as a Vec<Descriptor, String>
#[serde_as]
#[derive(Debug, Serialize, Deserialize)]
struct Todos(#[serde_as(as = "Vec<(_, _)>")] BTreeMap<Descriptor, String>);

// Describes the attributes of a todo. Indices should be unique
// when used in the todos struct
#[derive(Eq, PartialEq, Debug, Clone, Serialize, Deserialize)]
struct Descriptor {
    priority: usize,
    id: usize,
    group: Option<String>,
}

// Format looks like:
// (index) (group) (***)
// where number of stars indicates priority
// Index is green, group is yellow, and priority is bold red
impl Display for Descriptor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Descriptor {
            priority,
            id,
            group,
        } = self;
        // stars indicate priority
        let stars = match (*priority).clamp(0, 3) {
            0 => " ".into(),
            x => format!(" ({})", "*".repeat(x)),
        };

        // Format the part of the output determining the group
        let group = if let Some(group) = group {
            format!(" ({group})")
        } else {
            "".into()
        };

        write!(
            f,
            "{}",
            PRINTER
                .green(&format!(" ({id})"))
                .yellow(group)
                .bred(stars)
                .inner()
        )?;
        Ok(())
    }
}

// Order descriptors so we can get a sensible printing order.
// Order first consider priority, then group, then id
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
        // lower id = higher place on list
        match self.id.cmp(&other.id) {
            Ordering::Equal => {}
            Ordering::Less => {
                return Ordering::Greater;
            }
            Ordering::Greater => {
                return Ordering::Less;
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
