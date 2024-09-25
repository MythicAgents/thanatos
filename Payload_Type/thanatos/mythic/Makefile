### Variable definitions ###
GO		= go
GOFMT	= gofmt
GOLINT	= golangci-lint
GOFLAGS	=
PROTOC	= protoc

protobuf_srcdir := ../protobuf
protobuf_outdir := pb
protobuf_srcs := commands/exit.proto \
				 commands/sleep.proto \
				 config/config.proto \
				 msg/checkin.proto \
				 msg/mythic.proto \
				 msg/tasking.proto

protobuf_out := $(protobuf_srcs:%.proto=$(protobuf_outdir)/%.pb.go)
protobuf_deps := $(addsuffix .d,$(protobuf_out))

gomod != sed -nr 's/^module (.*)$$/\1/p' go.mod
gopkgdeps = $(patsubst $(gomod)/%,%,$(filter $(gomod)/%,$(shell $(GO) list -f '{{join .Deps " "}}' ./$(1))))
gosrcdeps = $(sort $(filter-out %_test.go,$(foreach pkg,$(call gopkgdeps,$(1)),$(wildcard $(pkg)/*.go))))

targets := server genconfig
godeps := $(targets:%=cmd/%/main.go.d)
deps := $(godeps) $(protobuf_deps)

### Invoked targets ###
.PHONY: all
all: server genconfig ## Build the 'server' and 'genconfig' targets. Supports parallelization

.PHONY: checkformat
checkformat: ## Check workspace code formatting
	$(GOFMT) -l -d . | diff -u /dev/null -

.PHONY: ci
ci: checkformat lint test ## Run CI workflow. Supports parallelization

.PHONY: coverage
coverage: | $(protobuf_out) ## Generate a code coverage report for tests
	@mkdir -p coverage/html
	$(GO) test -coverprofile coverage/tests.gocov
	$(GO) tool cover -html coverage/tests.gocov -o coverage/html/index.html

.PHONY: format
format: ## Format the workspace code
	$(GOFMT) -w .

genconfig: cmd/genconfig/main.go $(protobuf_out) ## Build the configuration generator cli

.PHONY: lint
lint: $(protobuf_out) ## Lint the workspace code
	$(GOLINT) run

.PHONY: protobuf
protobuf: $(protobuf_out) ## Build all protobuf files. Supports parallelization

server: cmd/server/main.go | $(protobuf_out) ## Build the Mythic server binary

.PHONY: test
test: $(protobuf_out) ## Run tests
	$(GO) test -v ./...

### Helper targets ###
$(protobuf_outdir)/%.pb.go: $(protobuf_srcdir)/%.proto $(protobuf_outdir)/%.pb.go.d | $(protobuf_outdir)
	$(PROTOC) -I$(protobuf_srcdir) --dependency_out=$(protobuf_outdir)/$*.pb.go.d --go_out=$(protobuf_outdir) --go_opt=paths=source_relative $<

%: cmd/%/main.go cmd/%/main.go.d
	@printf "$@: $(wildcard cmd/$*/*.go) $(call gosrcdeps,cmd/$*)\n" > cmd/$*/main.go.d ; \
		printf "$(wildcard cmd/$*/*.go) $(call gosrcdeps,cmd/$*) " | sed 's, ,:\n,g' >> cmd/$*/main.go.d
	$(GO) build $(GOFLAGS) -o $@ ./$(<D)

$(protobuf_outdir): ; @mkdir -p $@

$(deps):

#include $(wildcard $(deps))

.PHONY: debug
debug:
	@printf "$(call goprotodeps,cmd/server)\n"
	@printf "$(call gosrcdeps,cmd/server)\n"

### Housekeeping targets ###
.PHONY: clean
clean: ## Clean built artifacts
	$(RM) -r $(targets) $(protobuf_outdir) $(godeps)

.PHONY: help
help: ## Display this help menu
	@grep -hE '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[32m%-15s\033[0m- %s\n", $$1, $$2}'
