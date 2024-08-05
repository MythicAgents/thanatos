### Variable definitions ###
GO		= go
GOFMT	= gofmt
GOLINT	= golangci-lint
PROTOC	= protoc
GOMOD	:= github.com/MythicAgents/thanatos
GOFLAGS	?=

proto_outdir := proto
proto_srcdir := protobuf

### Protobuf sources ###
proto_srcs := \
	config/config.proto \
	msg/checkin/checkin.proto

proto_out := $(proto_srcs:%.proto=$(proto_outdir)/%.pb.go)


### Invoked targets ###
.PHONY: all
all: server genconfig ## Build the 'server' and 'genconfig' targets

.PHONY: checkformat
checkformat: ## Check workspace code formatting
	$(GOFMT) -l -d . | diff -u /dev/null -

.PHONY: ci
ci: checkformat lint test ## Run CI workflow

.PHONY: coverage
coverage: ## Generate a code coverage report for tests
	@mkdir -p coverage/html
	$(GO) test -coverprofile coverage/tests.gocov
	$(GO) tool cover -html coverage/tests.gocov -o coverage/html/index.html

.PHONY: format
format: ## Format the workspace code
	$(GOFMT) -w .

.PHONY: genconfig
genconfig: cmd/genconfig/main.go $(proto_out) ## Build the standalone configuration generator
	$(GO) build $(GOFLAGS) -o $@ ./cmd/genconfig

.PHONY: lint
lint: $(proto_out) ## Lint the workspace code
	$(GOLINT) run

.PHONY: protobuf
protobuf: $(proto_out) ## Build the protobuf files

.PHONY: server
server: $(proto_out) ## Build the Mythic server binary
	$(GO) build $(GOFLAGS) -o $@ ./cmd/server

.PHONY: test
test: $(proto_out) ## Run tests
	$(GO) test -v ./...

### Helper targets ###
# Pattern rule protobuf files
$(proto_outdir)/%.pb.go: $(proto_srcdir)/%.proto
	@mkdir -p $(dir $@)
	$(PROTOC) -I$(proto_srcdir) --go_out=$(proto_outdir) --go_opt=paths=source_relative --go_opt=M$*.proto=$(GOMOD)/$(@D) $<


### Housekeeping targets ###
.PHONY: clean
clean: ## Clean built artifacts
	$(RM) -rv $(proto_outdir) server genconfig

.PHONY: help
help: ## Print this help output
	@grep -hE '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[32m%-15s\033[0m- %s\n", $$1, $$2}'
