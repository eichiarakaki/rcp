# Makefile for rcopy
# Usage:
#   make install          # Installs to /usr/local/bin (may need sudo)
#   make install PREFIX=$HOME/.local
#   make uninstall
#   make uninstall PREFIX=$HOME/.local

PREFIX ?= /usr/local
BIN_DIR := $(PREFIX)/bin
BINARY := target/release/rcopy

.PHONY: all build install uninstall help

all: build

build:
	cargo build --release

install: build
	@echo "Installing rcopy to $(BIN_DIR)..."
	@mkdir -p $(BIN_DIR)
	@install -m 755 $(BINARY) $(BIN_DIR)/rcopy
	@echo "rcopy installed successfully to $(BIN_DIR)/rcopy"
	@echo "   You can now run: rcopy --help"

uninstall:
	@echo "Removing rcopy from $(BIN_DIR)..."
	@rm -f $(BIN_DIR)/rcopy
	@echo "rcopy has been uninstalled from $(BIN_DIR)"

help:
	@echo "rcopy Makefile"
	@echo ""
	@echo "Targets:"
	@echo "  make build           Build release binary"
	@echo "  make install         Install to $(PREFIX)/bin (default)"
	@echo "  make install PREFIX=~/.local   Install to user directory"
	@echo "  make uninstall       Remove installed binary"
	@echo ""
	@echo "Examples:"
	@echo "  sudo make install"
	@echo "  make install PREFIX=\$$HOME/.local"
	@echo "  make uninstall PREFIX=\$$HOME/.local"
