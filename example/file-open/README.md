# file-open

A small CLI that opens and reads text and binary (PNG) files asynchronously with [Tokio](https://tokio.rs),
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
  png-info    Show the size and colour format of a PNG image
  help        Print this message or the help of the given subcommand(s)
```

### Examples

> ### Command
> ```bash
> cargo run -p file-open -- open example/file-open/Cargo.toml
> ```
>
>> #### Result:
>> ```text
>> opened example/file-open/Cargo.toml
>> ```

> ### Command
> ```bash
> cargo run -p file-open -- read-lines example/file-open/Cargo.toml
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
>> file not found: nope.txt [example/file-open/src/main.rs:47:16]
>> ```

> ### Command
> ```bash
> cargo run -p file-open -- png-info example/file-open/assets/rgba-16bit-gradient.png
> ```
>
>> #### Result
>> ```text
>> 100 x 10 px, 16-bit RGBA
>> ```

> ### Command
> ```bash
> cargo run -p file-open -- png-info example/file-open/assets/not-a-png.png
> ```
>
>> #### Result
>> ```text
>> not a PNG file: example/file-open/assets/not-a-png.png [example/file-open/src/main.rs:69:43]
>> ```

On error, the message is printed to stderr and the process exits with code `1`.

## Layout

| File                   | Purpose                                                                    |
|------------------------|----------------------------------------------------------------------------|
| `src/main.rs`          | Entry point. Parses the CLI, dispatches each command to a handler fn.      |
| `src/cli/mod.rs`       | `Cli` — the top-level clap parser.                                         |
| `src/cli/command.rs`   | `Command` — the subcommands (`open`, `read-text`, `read-lines`, `png-info`). |
| `src/file/mod.rs`      | Async `open` and `read_prefix` (read the first *n* bytes of a file).       |
| `src/file/text.rs`     | Async `read_text` and `read_lines`.                                        |
| `src/file/png/info.rs` | `parse` — pure PNG header parser (no I/O), and the `Info` it returns.      |
| `src/error/mod.rs`     | `Error` enum and `Error::from_io`, which maps `io::ErrorKind` to variants. |
| `src/error/result.rs`  | `Result<T>` alias, re-exported as `FileResult<T>`.                         |
| `assets/`              | Sample PNGs for trying `png-info` (see [Assets](#assets)).                 |

## Errors

`std::io::Error`s are converted by `Error::from_io(source, path, location)`:

| `io::ErrorKind`    | Variant            | Keeps original `io::Error`? |
|--------------------|--------------------|-----------------------------|
| `NotFound`         | `NotFound`         | No                          |
| `PermissionDenied` | `PermissionDenied` | No                          |
| anything else      | `Io`               | Yes, as `#[source]`         |

`Png` is not an I/O error: it's raised by `png-info` when the bytes read
don't start with a valid PNG signature and `IHDR` chunk (including files too
short to contain one).

Every variant carries the file `path` and an `ErrorLocation` (`[file:line:column]`).

## Assets

`assets/` holds small generated PNGs — one per PNG colour type — plus two
invalid files for the error path.

| File                       | Contents                                 | `png-info` output                       |
|----------------------------|------------------------------------------|-----------------------------------------|
| `gray-1bit-checker.png`    | 1-bit checkerboard                       | `32 x 32 px, 1-bit grayscale`           |
| `gray-8bit-gradient.png`   | black → white gradient                   | `64 x 32 px, 8-bit grayscale`           |
| `gray-alpha-8bit-fade.png` | radial fade to transparent               | `48 x 48 px, 8-bit grayscale + alpha`   |
| `indexed-8bit-stripes.png` | 4-colour palette stripes                 | `40 x 20 px, 8-bit indexed (palette)`   |
| `rgb-8bit-gradient.png`    | red / green / blue gradient              | `64 x 64 px, 8-bit RGB`                 |
| `rgba-8bit-fade.png`       | blue fading to transparent               | `32 x 32 px, 8-bit RGBA`                |
| `rgba-16bit-gradient.png`  | 16-bit-per-channel gradient              | `100 x 10 px, 16-bit RGBA`              |
| `not-a-png.png`            | plain text with a `.png` extension       | `not a PNG file: ...`                   |
| `truncated.png`            | valid signature, cut off after 20 bytes  | `not a PNG file: ...`                   |

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
