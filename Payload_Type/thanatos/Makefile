### Invoked targets ###
.PHONY: all
all: mythic agent ## Build both the agent and mythic targets

.PHONY: agent
agent: ## Build the Agent portion of the project
	@$(MAKE) -C agent

.PHONY: mythic
mythic: ## Build the Mythic portion of the project
	@$(MAKE) -C mythic

.PHONY: ci agent/ci mythic/ci
ci: mythic/ci agent/ci ## Run the CI targets for the Agent and Mythic code.

mythic/ci:
	@$(MAKE) -C mythic ci

agent/ci:
	@$(MAKE) -C agent ci

### Housekeeping targets ###
.PHONY: clean agent/clean mythic/clean
clean: agent/clean mythic/clean ## Clean built artifacts

mythic/clean:
	@$(MAKE) -C mythic clean

agent/clean:
	@$(MAKE) -C agent clean

.PHONY: help
help: ## Display this help menu
	@echo -e "Top level Makefile for general housekeeping. All targets support parallelization with \`make -j\`.\n"
	@grep -hE '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[32m%-15s\033[0m- %s\n", $$1, $$2}'
