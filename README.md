# todo
A little todo tracker in your terminal (with pretty colors) `:)`

Making a todo is easy:

```
$ todo + "this is a todo"
```

We can also add priority and a group:

```
$ todo + "this todo has priority 2 and is in group hello" -pp -g hello
```

Here's how we can list todos:

```
$ todo l
this todo has priority 2 and is in group hello (1) (hello) (**)
this is a todo (2)
```
Todos are listed in order of priority. Within each priority class, tasks in the
same group are grouped together.
The numbers in parentheses allow us to access each todo. For example, deleting
a todo looks like:
```
$ todo - 1
Finished todo (2): this is a todo :)
```

## Installation
Clone this repository, run `cargo build --release`, and move the resulting
binary somewhere on your `$PATH` with `cp -i target/release/todo {somewhere
on $PATH}`.

## `--help`
```
$ todo -h
Usage: todo <COMMAND>

Commands:
  delete  Delete a todo [aliases: d, -]
  add     Add a todo [aliases: a, +]
  list    List all todos [aliases: l]
  edit    Edit the priority and group of a command using the same syntax as adding a command [aliases: e]
  help    Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help information
```

## License

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
