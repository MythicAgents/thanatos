TARGET    := thanatos
SOURCES   := $(wildcard ./**.go)
MODULE    ?= ./...
GOARGS    =

.PHONY: help
help: # Print out this help info
	@awk 'match($$0, /^([a-zA-Z-]+):.*(#.*)/, m) { print m[1] ":", m[2] }' Makefile

.PHONY: list
list: # List all the targets
	@awk 'match($$0, /^([a-zA-Z-]+):/, m) { print m[1] }' Makefile

.PHONY: clean
clean: # Remove the built server code
	rm -vf $(TARGET)

build: $(TARGET) # Build the Mythic server code

$(TARGET): $(SOURCES)
	go build -o $@

.PHONY: test
test: # Run the Go tests. Set 'GOARGS' to add additional "go test" arguments. Set 'MODULE' to the path of the package to test
	go test $(GOARGS) $(MODULE)

.PHONY: test-verbose
test-verbose: # Run the Go tests but in "verbose" mode
	@$(MAKE) GOARGS="-v" test

.PHONY: test-mockbuild
test-mockbuild: # Run all of the build tests with mock handlers
	@$(MAKE) GOARGS="$(GOARGS) -run \"^TestPayloadMockBuild\$\"" MODULE=./builder test

.PHONY: test-fullbuild
test-fullbuild: # Run all of the build tests
	@$(MAKE) GOARGS="$(GOARGS) -run \"^TestPayloadFullBuild\$\"" MODULE=./builder test
