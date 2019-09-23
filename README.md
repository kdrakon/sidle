# sidle ![GitHub release (latest by date)](https://img.shields.io/github/v/release/kdrakon/sidle)

**sidle** is a CLI tool that helps with directory and file selection.

For example, instead of
```bash
$ cd ../../../
```

or

```bash
$ cd ..
$ cd ..
$ cd ..
```

you can setup an alias to do this:

```bash
$ alias sd='sidle -o /tmp/sidle_path && cd $(cat /tmp/sidle_path)'
$ sd
```

[![asciicast](https://asciinema.org/a/qbrJWI1KrRquu7sX7HD9QK6f0.svg)](https://asciinema.org/a/qbrJWI1KrRquu7sX7HD9QK6f0)

## Usage
```
USAGE:
    sidle [FLAGS] [OPTIONS] [path]

FLAGS:
        --files-selectable    Allows files, in addition to directories, to be selected as output
    -h, --help                Prints help information
    -V, --version             Prints version information

OPTIONS:
    -o <output>        Where to write the final path chosen. Defaults to the file 'sidle_path' in a temp directory

ARGS:
    <path>    The path to start from. Defaults to current directory.
```

### Controls
- `ESC` or `q` key to quit without writing to output
- `Left/Right` keys to move between directories
- `Up/Down` keys to move up and down files and directories
- `PgUp/PgDown` keys to move up and down by 10 items
- `Enter/Return/`⏎ key to select the highlighted item
- `.` key to select the current directory

