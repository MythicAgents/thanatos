CARGO = cargo
GENCONFIG_EXE = ../genconfig

JSON_CONFIG ?= config.json

.PHONY: all
all: config ## Build all targets

.PHONY: config
config: .config ## Build the serialized config from the config.json file

.config: $(GENCONFIG_EXE)
	$(GENCONFIG_EXE) -i $(JSON_CONFIG) -o .config

$(GENCONFIG_EXE):
	$(MAKE) -C $(@D) genconfig

.PHONY: clean
clean: ## Clean built artifacts
	$(CARGO) clean
	$(RM) -v .config

.PHONY: help
help: ## Print this help output
	@grep -hE '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[32m%-11s\033[0m- %s\n", $$1, $$2}'
