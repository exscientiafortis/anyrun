# Shell

Run shell commands.

## Usage

Type in `<prefix><command>`, where `<prefix>` is the configured prefix (default in [Configuration](#Configuration)) and `<command>` is the command you want to run.

## Configuration

```ron
// <Anyrun config dir>/shell.ron
Config(
  prefix: ":sh",
  // Override the shell used to launch the command
  shell: None,
  // None to disable history (default)
  // note the double parens
  history: Some((
    capacity: 100,
    )),
)
```
