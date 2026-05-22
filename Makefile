.PHONY: all build test fmt lint audit ci hooks help

D = ./scripts/dev.sh

all: lint test build

build:
	$(D) cargo build --workspace --exclude rr-tauri

build-release:
	$(D) cargo build --release --package rr-cli

test:
	$(D) cargo test --workspace --exclude rr-tauri --locked

fmt:
	$(D) cargo fmt --all --check

lint: fmt
	$(D) cargo clippy --workspace --exclude rr-tauri -- -D warnings

audit:
	$(D) cargo deny check advisories bans licenses sources

ci: fmt lint test audit build-release

hooks:
	git config core.hooksPath .githooks
	@echo "✓ hooks installés"

help:
	@echo "Usage: make <target>"
	@echo ""
	@echo "  build       Compile le workspace (sans rr-tauri)"
	@echo "  build-release Compile rr-cli en release"
	@echo "  test        Lance tous les tests"
	@echo "  fmt         Vérifie le formatage"
	@echo "  lint        fmt + clippy"
	@echo "  audit       cargo-deny (advisories, licences, bans)"
	@echo "  ci          fmt + lint + test + audit + build-release"
	@echo "  hooks       Installe le pre-commit hook"
