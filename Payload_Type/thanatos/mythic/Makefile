# Go compiler
GO = go

# Go module base name
GOMOD := github.com/MythicAgents/thanatos

# Protobuf compiler
PROTOC = protoc

# List of targets to build for `make all`
all_targets := server genconfig

# Output directory for built protobufs
proto_outdir := proto

# Directory with protobuf sources
proto_srcdir := protobuf

define create-proto-target
	$1: $(patsubst $(proto_srcdir)/%.proto,$(proto_outdir)/%.pb.go,$(wildcard $(proto_srcdir)/$2/*.proto))
endef

.PHONY: all
all: $(all_targets) ## Build all targets

.PHONY: protobuf
protobuf: $(proto_srcs:%.proto=$(proto_outdir)/%.pb.go) ## Build all protobuf files

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

# Include protobuf sources
-include protobuf/sources.mk

# Include the cmd make modules
-include cmd/*/*.mk

# Include pkg proto make modules
-include pkg/*/proto.mk
