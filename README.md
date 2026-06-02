# rcopy

**rcopy** is a simple and fast command-line tool that recursively collects the contents of files and directories, merges them into a single block, and copies everything to your clipboard with clean path headers.

It is especially useful when you want to quickly share code context with LLMs (ChatGPT, Claude, Grok, etc.), create code reviews, or paste multiple files together.

## Features

- Recursively walks directories
- Supports excluding file types (`.go`, `.rs`, `.h`, etc.)
- Supports excluding specific files or directories
- Outputs clean formatted content with file paths as headers
- Works on **Wayland** (`wl-copy`) and **X11** (`xclip` / `xsel`)
- Handles very large projects (>100,000 lines)
- Cross-platform (Linux, macOS, Windows)
- Simple installation with `make` or Nix flakes

## Installation

### 1. Using Nix (Recommended on NixOS)

```bash
nix profile install .
nix run . -- --help
```

Or enter a development shell:

```bash
nix develop
```

### 2. Using Makefile

```bash
# System-wide (requires sudo)
sudo make install

# User-local install (recommended)
make install PREFIX=$HOME/.local
```

### 3. From Source

```bash
cargo build --release
sudo cp target/release/rcopy /usr/local/bin/
```

## Uninstall

```bash
sudo make uninstall
# or
make uninstall PREFIX=$HOME/.local
```

## Usage

```bash
rcopy [OPTIONS] <PATHS>...
```

### Examples

```bash
rcopy ./src ./README.md Cargo.toml

rcopy --copy ./dir1 ./dir2 example.go example.txt \
  --exclude-file-types .go .h \
  --exclude ./dir3 ./file.c
```

## Command Line Options

| Option                    | Description                                      |
|---------------------------|--------------------------------------------------|
| `--copy`                  | Optional flag                                    |
| `<paths>...`              | Files and/or directories                         |
| `--exclude-file-types`    | File extensions to exclude                       |
| `--exclude`               | Specific paths to exclude                        |

## Output Format

```
dir1/asd.c:
#include <stdio.h>
...

dir1/b.rs:
fn main() { ... }

example.go:
package main
...

example.txt:
Hello world
```

## Notes

- Hidden files are included by default.
- Symbolic links are not followed.
- Supports very large projects (128 MiB buffer).
- Requires `wl-copy` (Wayland) or `xclip`/`xsel` (X11).
