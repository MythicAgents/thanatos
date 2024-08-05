GO = go
PROTOC = protoc

proto_srcs := \
	config/config.proto \
	msg/checkin/checkin.proto


GOMOD := github.com/MythicAgents/thanatos

proto_outdir := proto
proto_srcdir := protobuf
proto_out := $(proto_srcs:%.proto=$(proto_outdir)/%.pb.go)


.PHONY: all
all: server genconfig ## Build all targets

.PHONY: protobuf
protobuf: $(proto_out) ## Build all protobuf files

server: cmd/server/main.go $(proto_out) ## Build the server component
	$(GO) build -o $@ ./cmd/server

genconfig: cmd/genconfig/main.go $(proto_out) ## Build the config builder
	$(GO) build -o $@ ./cmd/genconfig

# Pattern rule for building a protobuf file
$(proto_outdir)/%.pb.go: $(proto_srcdir)/%.proto
	@mkdir -p $(dir $@)
	$(PROTOC) -I$(proto_srcdir) --go_out=$(proto_outdir) --go_opt=paths=source_relative --go_opt=M$*.proto=$(GOMOD)/$(@D) $<


.PHONY: clean
clean: ## Clean built artifacts
	$(RM) -rv $(proto_outdir) server genconfig

.PHONY: help
help: ## Print this help output
	@grep -hE '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[32m%-11s\033[0m- %s\n", $$1, $$2}'
