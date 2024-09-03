### Variable definitions ###
GO		= go
GOFMT	= gofmt
GOLINT	= golangci-lint
GOFLAGS	?=
JQ		= jq

PROTOC		= protoc
PROTO_OUT	= pb
PROTO_SRC	= ../protobuf

gomod := $(shell sed -nr 's/^module (.*)$$/\1/p' go.mod)

proto_mods := commands config msg
proto_srcs := $(foreach dir,$(proto_mods),$(wildcard $(PROTO_SRC)/$(dir)/*.proto))
proto_gen := $(proto_srcs:$(PROTO_SRC)/%.proto=$(PROTO_OUT)/%.pb.go)
proto_mkdeps := $(addsuffix .d,$(proto_gen))

gopkgs := $(shell $(GO) list ./... 2>/dev/null) $(addprefix $(gomod)/$(PROTO_OUT)/,$(proto_mods))
go_mkdeps := $(gopkgs:$(gomod)/%=%/.dep)
mkdeps := $(proto_mkdeps) $(go_mkdeps)

targets := server genconfig $(proto_gen)

### Invoked targets ###
.PHONY: all
all: server genconfig ## Build the 'server' and 'genconfig' targets. Supports parallelization

.PHONY: checkformat
checkformat: ## Check workspace code formatting
	$(GOFMT) -l -d . | diff -u /dev/null -

.PHONY: ci
ci: checkformat lint test ## Run CI workflow. Supports parallelization

.PHONY: coverage
coverage: ## Generate a code coverage report for tests
	@mkdir -p coverage/html
	$(GO) test -coverprofile coverage/tests.gocov
	$(GO) tool cover -html coverage/tests.gocov -o coverage/html/index.html

.PHONY: format
format: ## Format the workspace code
	$(GOFMT) -w .

.PHONY: genconfig
genconfig: cmd/genconfig/.dep ## Build the configuration generator cli
	$(GO) build $(GOFLAGS) -o $@ ./cmd/genconfig

.PHONY: lint
lint: $(protobuf) ## Lint the workspace code
	$(GOLINT) run

.PHONY: protobuf
protobuf: $(proto_targets) ## Build all protobuf files. Supports parallelization

.PHONY: server
server: cmd/server/.dep  ## Build the Mythic server binary
	$(GO) build $(GOFLAGS) -o $@ ./cmd/server

.PHONY: test
test: $(protobuf) ## Run tests
	$(GO) test -v ./...

### Helper targets ###
VPATH = $(PROTO_SRC)

# Pattern rule for building Golang protobuf files
$(PROTO_OUT)/%.pb.go $(PROTO_OUT)/%.pb.go.d: %.proto
	@mkdir -p $(dir $@)
	$(PROTOC) -I$(PROTO_SRC) --dependency_out=$(PROTO_OUT)/$*.pb.go.d --go_out=$(PROTO_OUT) --go_opt=paths=source_relative $<

%/.dep:
	@set -e; $(RM) $@; \
		echo -e "[DEPS]:\t$*" ; \
		echo -ne "$@ : $(wildcard $*/*.go) " > $@ ; \
		$(GO) list -json ./$* > $@.$$$$.json ; \
		$(JQ) -r '.Imports | select(. != null) | map(select(. | startswith("$(gomod)"))) | map(. | sub("$(gomod)/";"")) | map(. += "/.dep") | join(" ")' $@.$$$$.json >> $@ ; \
		$(RM) $@.$$$$.json

-include $(mkdeps)

### Housekeeping targets ###
.PHONY: clean
clean: ## Clean built artifacts
	$(RM) $(targets)

.PHONY: distclean
distclean:
	$(RM) -r $(targets) $(PROTO_OUT) $(mkdeps)

.PHONY: help
help: ## Display this help menu
	@grep -hE '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[32m%-15s\033[0m- %s\n", $$1, $$2}'
