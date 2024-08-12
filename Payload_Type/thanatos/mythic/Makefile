### Variable definitions ###
GO		= go
GOFMT	= gofmt
GOLINT	= golangci-lint
GOFLAGS	?=

PROTOC		= protoc
PROTO_OUT	= pb
PROTO_SRC	= ../protobuf

proto_pkgs := pkg/builder
proto_targets := $(addsuffix /pb,$(proto_pkgs))

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
genconfig: cmd/genconfig/main.go pkg/builder/pb ## Build the configuration generator cli
	$(GO) build $(GOFLAGS) -o $@ ./cmd/genconfig

.PHONY: lint
lint: $(protobuf) ## Lint the workspace code
	$(GOLINT) run

.PHONY: protobuf
protobuf: $(proto_targets) ## Build all protobuf files. Supports parallelization

.PHONY: server
server: cmd/server/main.go pkg/builder/pb ## Build the Mythic server binary
	$(GO) build $(GOFLAGS) -o $@ ./cmd/server

.PHONY: test
test: $(protobuf) ## Run tests
	$(GO) test -v ./...

### Helper targets ###
VPATH = $(PROTO_SRC)

# Pattern rule for building Golang protobuf files
$(PROTO_OUT)/%.pb.go: %.proto
	@mkdir -p $(dir $@)
	$(PROTOC) -I$(PROTO_SRC) --go_out=$(PROTO_OUT) --go_opt=paths=source_relative $<

### Housekeeping targets ###
.PHONY: clean
clean: ## Clean built artifacts
	$(RM) -rv $(PROTO_OUT) server genconfig libgenconfig.a

.PHONY: help
help: ## Display this help menu
	@grep -hE '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[32m%-15s\033[0m- %s\n", $$1, $$2}'

include $(addsuffix /protobuf.mk,$(proto_pkgs))
