GO = go
FLAGS =
GOMOD := github.com/MythicAgents/thanatos

PROTOC = protoc

go_src := $(wildcard *.go) $(wildcard ./*/*.go) $(wildcard ./*/*/*.go)
proto_src := $(wildcard proto/*.proto) $(wildcard proto/*/*.proto) $(wildcard proto/*/*/*.proto)

protopath := proto
genprotopath := pb

VPATH := $(protopath)

generated_protos := $(proto_src:$(protopath)/%.proto=pb/%.pb.go)

.PHONY: all
all: thanatos ## Build all targets

thanatos: $(go_src) $(generated_protos) ## Build the Mythic server code
	$(GO) build $(FLAGS) -o $@

.PHONY: protobuf
protobuf: $(generated_protos) ## Build protobuf files

$(genprotopath)/%.pb.go: %.proto
	@mkdir -p $(dir $@)
	$(PROTOC) --proto_path=$(protopath) --go_out=$(genprotopath) --go_opt=paths=source_relative --go_opt=M$*.proto=$(GOMOD)/$(@D) $<

.PHONY: clean
clean: ## Clean project
	$(RM) -rv pb thanatos

.PHONY: help
help: ## Print this help output
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[32m%s:\033[0m\n\t%s\n", $$1, $$2}'
