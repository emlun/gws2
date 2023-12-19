MAKEFLAGS = --warn-undefined-variables
SHELL = bash

.PHONY: help
help: ## Display this help.
	@awk 'BEGIN {FS = ":.*##"; printf "\nUsage:\n  make \033[36m<target>\033[0m\n"} /^[a-zA-Z_0-9-]+:.*?##/ { printf "  \033[36m%-15s\033[0m %s\n", $$1, $$2 } /^##@/ { printf "\n\033[1m%s\033[0m\n", substr($$0, 5) } ' $(MAKEFILE_LIST)

.PHONY: FORCE
FORCE: ;

##@ Install:

.PHONY: install
install: ~/.cargo/bin/gws ## Install gws into ~/.cargo/bin/gws

~/.cargo/bin/gws: FORCE
	@cargo install --path .

##@ Development:

.PHONY: fmt
fmt: ## Format the code
	@cargo fmt

.PHONY: test
test: ## Run the tests
	@cargo test

.PHONY: build
build: ## Build the project
	@cargo build

.PHONY: run
run: ## Run the project
	@cargo run
