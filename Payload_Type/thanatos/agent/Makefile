### Variable definitions ###
CARGO		= cargo
GENCONFIG	= ../mythic/genconfig
GRCOV		= grcov
GENHTML		= genhtml
JSON_CONFIG	?= config.json


### Invoked targets ###
.PHONY: all
all: binary library ## Build the 'binary' and 'library' targets

.PHONY: checkformat
checkformat: ## Check workspace code formatting
	$(CARGO) fmt --all -- --color always --check

.PHONY: ci
ci: checkformat lint test sanitizers ## Run ci workflow

.PHONY: config
config: .config ## Build the serialized config from the config.json file

.PHONY: coverage
coverage: .config ## Generate a code coverage report for tests
	@mkdir -p ./target/debug/coverage/html
	RUSTFLAGS='-Ccodegen-units=1 -Copt-level=0 -Cinstrument-coverage' $(CARGO) test --workspace --all-features
	$(GRCOV) . \
		-s . \
		--binary-path ./target/debug/ \
		-t lcov \
		--branch \
		--ignore-not-existing \
		--ignore "$HOME/.cargo/registry/*" \
		--ignore "*/target/*" \
		--ignore "**/build.rs" \
		-o ./target/debug/coverage/tests.lcov
	$(GENHTML) \
		-o ./target/debug/coverage/html \
		-f \
		--dark-mode \
		--show-details \
		--show-navigation \
		--highlight \
		--ignore-errors source \
		--legend \
		./target/debug/coverage/tests.lcov
	find . -name "default*.profraw" -exec rm {} \;

.PHONY: format
format: .config ## Format the workspace code
	cargo fmt --all

.PHONY: test
test: .config ## Run tests
	cargo test --color always --workspace --all-features

.PHONY: thanatos
thanatos: .config ## Build the thanatos core package
	$(CARGO) b -p thanatos_core

.PHONY: binary
binary: .config ## Build the thanatos binary executable
	$(CARGO) b -p thanatos_binary

.PHONY: library
library: .config ## Build the thanatos shared library
	$(CARGO) b -p thanatos_library

.PHONY: lint
lint: .config ## Lint the workspace code
	$(CARGO) clippy --workspace --color always --all-features --all-targets -- -D warnings

.PHONY: sanitizers
sanitizers: ## Run unsafe code tests with sanitizers
	@$(MAKE) SANITIZER=address sanitizer
	@$(MAKE) SANITIZER=memory sanitizer


### Helper targets ###
# Builds the serialized config
.config: $(GENCONFIG) $(JSON_CONFIG)
	$(GENCONFIG) -i $(JSON_CONFIG) -o .config

# Builds the configuration generator
$(GENCONFIG):
	$(MAKE) -C $(@D) genconfig

.PHONY: sanitizer
sanitizer: .config
	RUSTFLAGS='-Zsanitizer=$(SANITIZER)' $(CARGO) +nightly test -Zbuild-std --color always -p ffiwrappers --target x86_64-unknown-linux-gnu -- --all-features

### Housekeeping targets ###
.PHONY: clean
clean: ## Clean built artifacts
	$(CARGO) clean
	$(RM) -v .config

.PHONY: help
help: ## Print this help output
	@grep -hE '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[32m%-15s\033[0m- %s\n", $$1, $$2}'
