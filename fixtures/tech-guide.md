This guide covers how to set up and use a fictional command-line tool called `toolbox`.

# Getting Started with Toolbox

Toolbox is a general-purpose CLI for managing local development environments. It works on Linux, macOS, and Windows.

## Installation

You can install Toolbox using the following methods.

### Using a Package Manager

On macOS, run:

```bash
brew install toolbox
# this comment has a # character that should not be a header
```

On Linux with apt:

```bash
apt-get install toolbox
```

### Manual Installation

Download the binary from the releases page and place it in your `$PATH`.

```bash
# not a header
curl -L https://example.com/toolbox/latest -o toolbox
chmod +x toolbox
mv toolbox /usr/local/bin/
```

Verify the installation:

```bash
toolbox --version
```

## Configuration

Toolbox reads configuration from `~/.toolbox/config.toml`.

### Global Options

The following keys are supported at the top level:

```toml
# Global settings
log_level = "info"
cache_dir = "~/.toolbox/cache"
```

### Project Options

Per-project settings live in `.toolbox.toml` in your project root.

## Usage

Run `toolbox help` to see all available commands. The most common commands are listed below.

### Starting a Session

```bash
toolbox start --env development
```

### Stopping a Session

```bash
toolbox stop
```

### Running a Task

```bash
toolbox run build
```

## Troubleshooting

Common issues and their solutions.

### Command Not Found

Make sure `toolbox` is on your `$PATH`. Run `which toolbox` to check.

### Permission Denied

On Unix systems, ensure the binary is executable:

```bash
chmod +x $(which toolbox)
```

#### Checking Logs

Logs are written to `~/.toolbox/logs/`. Use `tail -f` to follow them in real time.

Inline code like `#config` in text should not be treated as a header.
