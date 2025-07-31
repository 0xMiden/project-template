.DEFAULT_GOAL := help

.PHONY: help
help:
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-30s\033[0m %s\n", $$1, $$2}'

# -- variables ------------------------------------------------------------------------------------

WARNINGS=RUSTDOCFLAGS="-D warnings"

# -- linting --------------------------------------------------------------------------------------

.PHONY: clippy
clippy: ## Runs Clippy with configs
	CLIPPY_CONF_DIR=configs cargo clippy --locked --all-targets --all-features --workspace -- -D warnings


.PHONY: fix
fix: ## Runs Fix with configs
	cargo fix --allow-staged --allow-dirty --all-targets --all-features --workspace


.PHONY: format
format: ## Runs Format using nightly toolchain
	cargo +nightly fmt --all -- --config-path configs/rustfmt.toml


.PHONY: format-check
format-check: ## Runs Format using nightly toolchain but only in check mode
	cargo +nightly fmt --all --check -- --config-path configs/rustfmt.toml


.PHONY: toml
toml: ## Runs Format for all TOML files
	taplo fmt -c configs/.taplo.toml


.PHONY: toml-check
toml-check: ## Runs Format for all TOML files but only in check mode
	taplo fmt -c configs/.taplo.toml --check --verbose

.PHONY: typos-check
typos-check: ## Runs spellchecker
	typos -c configs/_typos.toml

.PHONY: workspace-check
workspace-check: ## Runs a check that all packages have `lints.workspace = true`
	cargo workspace-lints


.PHONY: lint
lint: format fix clippy toml workspace-check ## Runs all linting tasks at once (Clippy, fixing, formatting, workspace)

# --- docs ----------------------------------------------------------------------------------------

.PHONY: doc
doc: ## Generates & checks documentation
	$(WARNINGS) cargo doc --all-features --keep-going --release --locked

.PHONY: book
book: ## Builds the book & serves documentation site
	mdbook serve --open docs

# --- testing -------------------------------------------------------------------------------------

.PHONY: test
test:  ## Runs all tests
	cargo nextest run --all-features --workspace

.PHONY: doc-test
doc-test: ## Runs doc tests
	cargo test --doc

# --- checking ------------------------------------------------------------------------------------

.PHONY: check
check: ## Check all targets and features for errors without code generation
	cargo check --all-features --all-targets --locked --workspace

# --- building ------------------------------------------------------------------------------------

.PHONY: build
build: ## Builds all crates and re-builds protobuf bindings for proto crates
	cargo build --locked --workspace

#### # TODO(template) below sections for binary ####

# --- installing ----------------------------------------------------------------------------------

.PHONY: install-mybinary
install-mybinary: ## Installs mybinary
	cargo install --path bin/mybinary --locked

# --- docker --------------------------------------------------------------------------------------

.PHONY: docker-build-mybinary
docker-build-mybinary: ## Builds the binary using Docker
	@CREATED=$$(date) && \
	VERSION=$$(cat bin/mybinary/Cargo.toml | grep -m 1 '^version' | cut -d '"' -f 2) && \
	COMMIT=$$(git rev-parse HEAD) && \
	docker build --build-arg CREATED="$$CREATED" \
        		 --build-arg VERSION="$$VERSION" \
          		 --build-arg COMMIT="$$COMMIT" \
                 -f bin/mybinary/Dockerfile \
                 -t miden-mybinary-image .

.PHONY: docker-run-mybinary
docker-run-mybinary: ## Runs mybinary as a Docker container
	docker volume create mybinary-db
	docker run --name miden-mybinary \
			   -p 57291:57291 \
               -v mybinary-db:/db \
               -d miden-mybinary-image