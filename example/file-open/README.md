# file-open

A small CLI that opens and reads text files asynchronously with [Tokio](https://tokio.rs),
using [clap](https://docs.rs/clap) for argument parsing and
[thiserror](https://docs.rs/thiserror) + [error-location](https://docs.rs/error-location)
for errors that report *where* they were triggered.

## Usage

Run from the workspace root. Everything after `--` is passed to the program.

```bash
cargo run -p file-open -- --help
```

```text
Usage: file-open <COMMAND>

Commands:
  open        Open a file and report whether it succeeded
  read-text   Print the whole file
  read-lines  Print the file line by line, with line numbers
  help        Print this message or the help of the given subcommand(s)
```

### Examples

> ### Command
> ```bash
> cargo run -p file-open -- open > example/file-open/Cargo.toml
> ```
>
>> #### Result:
>> ```text
>> opened example/file-open/Cargo.toml
>> ```

> ### Command
> ```bash
> cargo run -p file-open -- read-lines > example/file-open/Cargo.toml
> ```
>
>> #### Result
>> ```text
>>    1: [package]
>>    2: name           = "file-open"
>>    ...
>> ```

> ### Command
> ```bash
> cargo run -p file-open -- read-text nope.txt
> ```
>
>> #### Result
>> ```text
>> file not found: nope.txt > [example/file-open/src/main.rs:44:16]
>> ```

On error, the message is printed to stderr and the process exits with code `1`.

## Layout

| File                  | Purpose                                                                 |
|-----------------------|-------------------------------------------------------------------------|
| `src/main.rs`         | Entry point. Parses the CLI, dispatches each command to a handler fn.   |
| `src/cli/mod.rs`      | `Cli` — the top-level clap parser.                                      |
| `src/cli/command.rs`  | `Command` — the subcommands (`open`, `read-text`, `read-lines`).        |
| `src/file.rs`         | Async file operations: `open`, `read_text`, `read_lines`.               |
| `src/error/mod.rs`    | `Error` enum and `Error::from_io`, which maps `io::ErrorKind` to variants. |
| `src/error/result.rs` | `Result<T>` alias, re-exported as `FileResult<T>`.                      |

## Errors

`std::io::Error`s are converted by `Error::from_io(source, path, location)`:

| `io::ErrorKind`    | Variant            | Keeps original `io::Error`? |
|--------------------|--------------------|-----------------------------|
| `NotFound`         | `NotFound`         | No                          |
| `PermissionDenied` | `PermissionDenied` | No                          |
| anything else      | `Io`               | Yes, as `#[source]`         |

Every variant carries the file `path` and an `ErrorLocation` (`[file:line:column]`).

## Design notes

### Caller locations with async

`#[track_caller]` has no effect on an `async fn` in stable Rust
(see [rust-lang/rust#110011](https://github.com/rust-lang/rust/issues/110011)),
so the location would point inside `file.rs` instead of at the caller.

To work around this, each public function in `file.rs` is a plain `fn` marked
`#[track_caller]` that:

1. captures `Location::caller()` synchronously, then
2. returns the async work as `impl Future<Output = FileResult<T>>`.

```rust
#[track_caller]
pub(crate) fn read_text(path: impl AsRef<Path>) -> impl Future<Output = FileResult<String>> {
    let location = ErrorLocation::from(Location::caller());
    let path = path.as_ref().to_path_buf();
    async move { /* ... */ }
}
```

The same reason applies to closures: `.map_err(|e| ...)` does not inherit
`#[track_caller]`, so errors are built with an explicit `match` instead.

### Why `from_io` and not `impl From<io::Error>`

`From::from` only receives the `io::Error`, so it can't attach the file path.
A named constructor takes the path and location as extra arguments.

### Is async worth it here?

For a standalone CLI, sync `std::fs` would be just as fast (and `tokio::fs` runs
blocking calls on a thread pool under the hood). This example uses async to show
the pattern for code that lives inside an async runtime, where blocking file I/O
would stall the executor.
