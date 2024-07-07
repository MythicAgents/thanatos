genconfig: cmd/genconfig/main.go pkg/builder ## Build the standalone config generator
	$(GO) build $(GOMOD)/cmd/genconfig
