### Variable definitions ###
PROTOC	= protoc

proto_srcs := \
			  $(wildcard commands/*.proto) \
			  $(wildcard config/*.proto) \
			  $(wildcard msg/*.proto)

proto_go_out := $(proto_srcs:.proto=.pb.go)
proto_rs_out := $(proto_srcs:.proto=.rs)

### Invoked targets ###
.PHONY: all
all: golang rust ## Build the protobuf files for Golang and Rust

.PHONY: golang
golang: $(proto_go_out) ## Build protobuf files for Golang

.PHONY: rust
rust: $(proto_rs_out) ## Build protobuf files for Rust

### Helper targets ###
%.pb.go: %.proto
	$(PROTOC) -I. --go_out=. --go_opt=paths=source_relative $<

%.rs: %.proto
	$(PROTOC) -I. --rust_out=$(dir $@) $<


### Housekeeping targets ###
.PHONY: clean
clean: ## Clean built artifacts
	$(RM) -rv $(proto_go_out) $(proto_rs_out)

.PHONY: help
help: ## Print this help output
	@echo -e "Makefile used for testing protobuf builds with the Rust and Go plugins.\n"
	@grep -hE '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[32m%-15s\033[0m- %s\n", $$1, $$2}'
