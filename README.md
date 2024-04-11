# Sessionizer: A CLI Tool for Tmux Session Management

Sessionizer is a powerful command-line interface (CLI) tool designed to manage `tmux` sessions dynamically based on filesystem rules. It leverages a configuration file to define directories and session rules, and integrates with `fzf` for interactive selection, enhancing your workflow with `tmux`.

## Features

- **Dynamic Session Management**: Create and manage `tmux` sessions based on predefined directory rules.
- **Interactive Selection**: Utilize `fzf` for interactive session and directory selection.
- **Configuration Flexibility**: Define your session rules and directories through a YAML configuration file.

## Installation

Before you can use Sessionizer, ensure you have `tmux` and `fzf`, and `Rust` installed on your system as prerequisites.

1. **Install Tmux**: Follow the installation instructions for your operating system.
2. **Install Fzf**: Refer to the [official Fzf repository](https://github.com/junegunn/fzf) for installation guidelines.
3. **Install Rust**: Install Rust and Cargo using [rustup](https://rustup.rs/).

After setting up the prerequisites, clone the Sessionizer repository and build it using Cargo:

```sh
git clone https://example.com/sessionizer.git
cd sessionizer
cargo build --release
```

## Configuration

Sessionizer relies on a YAML configuration file to define its behavior. Here's a sample configuration:

```yaml
directories:
  - id: "unique-id-1"
    path: "/path/to/directory"
    mindepth: 1
    maxdepth: 1
    grep: ".*"
sessions:
  - "session-name-1"
env:
  - "VAR=value"
```

- **directories**: Defines the directories to be included or excluded from session management.
- **sessions**: Lists previously managed sessions.
- **env**: Specifies environment variables to be set in sessions.

## Usage

After configuring Sessionizer, you can manage your `tmux` sessions using the following commands:

### Initialize Configuration

To create a new configuration file or overwrite an existing one:

```sh
sessionizer config init [--force]
```

### Edit Configuration

To open the configuration file in your default editor:

```sh
sessionizer config edit
```

### Print Configuration

To display the current configuration:

```sh
sessionizer config print
```

### Add a Directory

To add a new directory for session management:

```sh
sessionizer directories add --path "/path/to/directory" [--mindepth 1] [--maxdepth 1] [--grep ".*"]
```

### Remove a Directory

To remove a directory from session management:

```sh
sessionizer directories remove --id "unique-id"
```

### List Directories

To list all configured directories:

```sh
sessionizer directories list
```

### Evaluate Directories

To evaluate and list directories based on the current configuration:

```sh
sessionizer directories evaluate
```

### Sessions Management

Sessionizer allows you to create, manage, and switch between `tmux` sessions based on your configured directories:

- **Create a New Session**: `sessionizer sessions new [--session "session-name"]`
- **Switch to a Session**: `sessionizer sessions go [--session "session-name"]`
- **List Session History**: `sessionizer sessions history`
- **Add a Session to History**: `sessionizer sessions add --session "session-name" [--set]`
- **Remove a Session**: `sessionizer sessions remove --session "session-name"`
- **Sync Sessions**: `sessionizer sessions sync [--reverse]`

## Advanced Usage

For more advanced use cases, such as scripting or integration with other tools, refer to the `--help` option for each command to explore all available flags and parameters.

## Contributing

Contributions to Sessionizer are welcome! Whether it's feature requests, bug reports, or pull requests, your input is valuable in making Sessionizer better for everyone.

## License

Sessionizer is released under the MIT License. See the LICENSE file for more details.

---

For more information, visit the [Sessionizer GitHub repository](https://github.com/cloudbridgeuy/sessionizer.git).
