CARGOARGS	?=
TARGET		?= x86_64-unknown-linux-gnu
CONFIG		?= $(PWD)/.config.bin
SOURCES		= $(wildcard ./**.rs)
SRC_CONFIG	?= $(PWD)/.config
PACKAGE		?=


.PHONY: help
help: # Print out this help info
	@awk 'match($$0, /^([a-zA-Z].*):.*(#.*)/, m) { print m[1] ":", m[2] }' Makefile

.PHONY: list
list: # List all the targets
	@awk 'match($$0, /^([a-zA-Z].*):/, m) { print m[1] }' Makefile

define newline


endef

define DEFAULT_CONFIG
UUID=00000000-0000-0000-0000-000000000000
INIT_OPTION=none
WORKING_HOURS_START=0
WORKING_HOURS_END=86400
CONNECTION_RETRIES=1
TLS_SELF_SIGNED=false
HTTP_CALLBACK_HOST=http://mythic
HTTP_CALLBACK_INTERVAL=10
HTTP_CALLBACK_JITTER=23
HTTP_CALLBACK_PORT=80
HTTP_KILLDATE=4070908800
HTTP_ENCRYPTED_EXCHANGE_CHECK=true
HTTP_HEADERS=eyJVc2VyLUFnZW50IjoidGVzdCJ9
HTTP_GET_URI=index
HTTP_POST_URI=data
HTTP_QUERY_PATH_NAME=q
endef


$(SRC_CONFIG):
	@echo -e '$(subst $(newline),\n,${DEFAULT_CONFIG})' > $(SRC_CONFIG)

$(CONFIG): $(SRC_CONFIG)
	SRC_CONFIG=$(SRC_CONFIG) CONFIG=$(CONFIG) cargo build -p genconfig

.config:
	@echo -e '$(subst $(newline),\n,${DEFAULT_CONFIG})' > $(SRC_CONFIG)

.config.bin: .config
	SRC_CONFIG=$(SRC_CONFIG) CONFIG=$(CONFIG) cargo build -p genconfig

.PHONY: test
test: # Run all of the tests
	cargo test --workspace --target $(TARGET) $(CARGOARGS)

.PHONY: genconfig
genconfig: $(CONFIG) # Generate a configuration file for building the agent

.PHONY: clean
clean: # Remove the built binaries
	cargo clean

.PHONY: distclean
distclean: # Remove the build configuration
	rm -vf $(CONFIG)

.PHONY: http
http: # Build the HTTP version of the thanatos agent
	@$(MAKE) _http PACKAGE=thanatos_http_binary
	@$(MAKE) _http PACKAGE=thanatos_http_cdylib

.PHONY: http_binary
http_binary: # Build the HTTP version of the thanatos agent as a binary
	@$(MAKE) _http PACKAGE=thanatos_http_binary

.PHONY: http_library
http_library: # Build the HTTP version of the thanatos agent as a library
	@$(MAKE) _http PACKAGE=thanatos_http_cdylib

# Hidden build targets for abstracting the other build targets
.PHONY: _http
_http: $(CONFIG)
	CONFIG=$(CONFIG) cargo build -p $(PACKAGE) --target $(TARGET) $(CARGOARGS)
