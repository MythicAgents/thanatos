### Variable definitions ###
GO		= go
GOFMT	= gofmt
GOLINT	= golangci-lint
GOFLAGS	?=
PROTOC	= protoc


depdir := .deps

protobuf_srcdir := ../protobuf
protobuf_gendir := pb
protobuf_subdirs := commands config msg
protobuf_depdir := $(depdir)/$(protobuf_gendir)
protobuf_depdirs := $(protobuf_depdir) $(addprefix $(protobuf_depdir)/,$(protobuf_subdirs))

protobuf_srcs := $(foreach subdir,$(protobuf_subdirs),$(wildcard $(protobuf_srcdir)/$(subdir)/*.proto))
protobuf_gen := $(protobuf_srcs:$(protobuf_srcdir)/%.proto=$(protobuf_gendir)/%.pb.go)
protobuf_mkdeps := $(protobuf_gen:$(protobuf_gendir)/%=$(protobuf_depdir)/%.d)

gomod := $(shell sed -nr 's/^module (.*)$$/\1/p' go.mod)
gopkgdeps = $(patsubst $(gomod)/%,%,$(filter $(gomod)/%,$(shell $(GO) list -f '{{join .Deps " "}}' ./$(1))))
goprotodeps = $(filter $(filter $(protobuf_gendir)/%,$(call gopkgdeps,$(1)))/%,$(protobuf_gen))

targets := server genconfig
targets_depdir := $(depdir)/cmd
targets_mkdeps := $(targets:%=$(targets_depdir)/%.mk)

depdirs := $(depdir) $(targets_depdir) $(protobuf_depdirs)
mkdeps := $(targets_mkdeps) $(protobuf_mkdeps)

### Invoked targets ###
.PHONY: all
all: server genconfig ## Build the 'server' and 'genconfig' targets. Supports parallelization

.PHONY: checkformat
checkformat: ## Check workspace code formatting
	$(GOFMT) -l -d . | diff -u /dev/null -

.PHONY: ci
ci: checkformat lint test ## Run CI workflow. Supports parallelization

.PHONY: coverage
coverage: $(protobuf_gen) ## Generate a code coverage report for tests
	@mkdir -p coverage/html
	$(GO) test -coverprofile coverage/tests.gocov
	$(GO) tool cover -html coverage/tests.gocov -o coverage/html/index.html

.PHONY: format
format: ## Format the workspace code
	$(GOFMT) -w .

genconfig: cmd/genconfig/main.go ## Build the configuration generator cli

.PHONY: lint
lint: $(protobuf_gen) ## Lint the workspace code
	$(GOLINT) run

.PHONY: protobuf
protobuf: $(protobuf_gen) ## Build all protobuf files. Supports parallelization

server: cmd/server/main.go ## Build the Mythic server binary

.PHONY: test
test: $(protobuf_gen) ## Run tests
	$(GO) test -v ./...

### Helper targets ###
$(protobuf_gendir)/%.pb.go: $(protobuf_srcdir)/%.proto $(protobuf_depdir)/%.pb.go.d | $(protobuf_gendir)
	@mkdir -p $(protobuf_depdir)/$(*D)
	$(PROTOC) -I$(protobuf_srcdir) --dependency_out=$(protobuf_depdir)/$*.pb.go.d --go_out=$(protobuf_gendir) --go_opt=paths=source_relative $<

%: cmd/%/main.go $(targets_depdir)/%.mk | $(targets_depdir)
	@printf "$@ $(targets_depdir)/$*.mk : $(call goprotodeps,$(<D)) $(<D) $(call gopkgdeps,$(<D))" > $(targets_depdir)/$*.mk
	$(GO) build $(GOFLAGS) -o $@ ./$(<D)

$(protobuf_gendir) $(targets_depdir): ; @mkdir -p $@

$(mkdeps):
$(protobuf_gen):

include $(mkdeps)

.PHONY: debug
debug:
	@printf "$(call goprotodeps,cmd/server)"

### Housekeeping targets ###
.PHONY: clean
clean: ## Clean built artifacts
	$(RM) $(targets) $(mkdeps) $(protobuf_gen)

.PHONY: distclean
distclean:
	$(RM) -r $(targets) $(depdir) $(protobuf_gendir)

.PHONY: help
help: ## Display this help menu
	@grep -hE '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[32m%-15s\033[0m- %s\n", $$1, $$2}'
