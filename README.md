# rcp

**rcp** is a small and fast command-line tool that recursively collects files and directories, merges their contents into a single block, and copies the result to your clipboard with clean path headers.

It is built for quickly sharing project context with LLMs such as ChatGPT, Claude, Grok, or any other coding assistant. It is also useful for code reviews, debugging sessions, and pasting multiple source files together without manually opening each one.

## Features

* Recursively walks files and directories
* Merges file contents into a single clipboard payload
* Adds clean path headers before each file
* Supports explicit ignore rules with `--ignore` / `-i`
* Includes sensible default ignores for build outputs, caches, dependencies, lock files, binaries, media, and generated artifacts
* Supports disabling default ignores with `--no-default-ignores`
* Supports excluding additional file extensions with `--exclude-file-types`
* Does not follow symbolic links
* Works on Wayland via `wl-copy`
* Falls back to X11 via `xclip` or `xsel`
* Handles large projects with a large preallocated buffer

## Installation

### Using Nix

```bash
nix profile install .
```

Run directly:

```bash
nix run . -- --help
```

Development shell:

```bash
nix develop
```

### Using Makefile

System-wide install:

```bash
sudo make install
```

User-local install:

```bash
make install PREFIX=$HOME/.local
```

### From Source

```bash
cargo build --release
sudo cp target/release/rcp /usr/local/bin/
```

## Uninstall

```bash
sudo make uninstall
```

Or for user-local installs:

```bash
make uninstall PREFIX=$HOME/.local
```

## Usage

```bash
rcp [OPTIONS] <PATHS>...
```

## Examples

Copy a source directory and a few project files:

```bash
rcp ./src ./README.md Cargo.toml
```

Copy the current project while ignoring extra paths:

```bash
rcp . -i secrets.toml -i notes/private.md
```

Ignore directories:

```bash
rcp . -i target -i node_modules -i .git
```

Ignore glob-style patterns:

```bash
rcp . -i "*.env" -i "*.lock" -i "dist/*.js"
```

Exclude file types manually:

```bash
rcp . --exclude-file-types .png .jpg .pdf
```

Disable all built-in default ignores:

```bash
rcp . --no-default-ignores
```

## Command Line Options

| Option                          | Description                                                |
| ------------------------------- | ---------------------------------------------------------- |
| `<PATHS>...`                    | Files and/or directories to collect                        |
| `--ignore <PATTERN>`            | Ignore a file, directory, path, or simple glob pattern     |
| `-i <PATTERN>`                  | Short form of `--ignore`                                   |
| `--exclude-file-types <EXT>...` | Exclude files by extension                                 |
| `--no-default-ignores`          | Disable built-in default ignore rules                      |
| `--copy`                        | Compatibility flag; clipboard copy is the default behavior |

## Ignore Rules

`rcp` supports repeated ignore rules:

```bash
rcp . -i target -i .git -i "*.lock"
```

Supported ignore forms:

| Pattern         | Meaning                                        |
| --------------- | ---------------------------------------------- |
| `.git`          | Ignore any path component named `.git`         |
| `target`        | Ignore any file or directory named `target`    |
| `*.lock`        | Ignore files ending in `.lock`                 |
| `result-*`      | Ignore names matching the wildcard             |
| `dist/*.js`     | Ignore relative paths matching the wildcard    |
| `src/generated` | Ignore a specific path and everything under it |

## Default Ignores

By default, `rcp` skips common files and directories that are usually useless when sending code context to an LLM.

Default ignored examples include:

```text
.git
target
node_modules
dist
build
result
result-*
.direnv
.devenv
.venv
venv
__pycache__
*.lock
*.log
*.tmp
*.png
*.jpg
*.pdf
*.zip
*.tar.gz
*.mp4
*.mp3
*.exe
*.so
*.dll
*.sqlite
*.parquet
*.ttf
*.woff2
```

To include everything, disable default ignores:

```bash
rcp . --no-default-ignores
```

## Output Format

```text
src/main.rs:
fn main() {
    println!("hello");
}

Cargo.toml:
[package]
name = "rcp"
version = "0.1.0"

README.md:
# rcp
...
```

## Clipboard Backends

`rcp` tries clipboard tools in this order:

1. `wl-copy` for Wayland
2. `xclip` for X11
3. `xsel` for X11

Install the appropriate clipboard backend for your environment.

### Wayland

```bash
sudo pacman -S wl-clipboard
```

```bash
nix profile install nixpkgs#wl-clipboard
```

### X11

```bash
sudo pacman -S xclip
```

or:

```bash
sudo pacman -S xsel
```

## Notes

* Hidden files are included unless ignored by default or via `-i`.
* Symbolic links are not followed.
* Binary and media files are ignored by default.
* Lock files are ignored by default.
* The clipboard payload is preallocated for large projects.
* This tool is primarily designed for Linux clipboard workflows.
