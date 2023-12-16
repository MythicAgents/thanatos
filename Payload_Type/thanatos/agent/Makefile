CARGOARGS	?=
TARGET		?= x86_64-unknown-linux-gnu
CONFIG		?= .config
SOURCES		:= $(wildcard ./**.rs)


ifneq (,$(wildcard $(CONFIG)))
	include $(CONFIG)
	export $(shell sed 's/=.*//' $(CONFIG))
endif


.PHONY: help
help: # Print out this help info
	@awk 'match($$0, /^([a-zA-Z\-\.]+):.*(#.*)/, m) { print m[1] ":", m[2] }' Makefile

.PHONY: list
list: # List all the targets
	@awk 'match($$0, /^([a-zA-Z][a-zA-Z\-\.]*):/, m) { print m[1] }' Makefile

define newline


endef

define DEFAULT_CONFIG
UUID=$(shell uuidgen)
CONNECTION_RETRIES=1
WORKING_HOURS_START=0
WORKING_HOURS_END=86400
HTTP_CALLBACK_JITTER=23
HTTP_CALLBACK_HOST=http://mythic
HTTP_CALLBACK_PORT=80
HTTP_GET_URI=index
HTTP_CALLBACK_INTERVAL=1
HTTP_KILLDATE=4070908800
HTTP_HEADERS=eyJVc2VyLUFnZW50IjoidGVzdCJ9
HTTP_POST_URI=data
HTTP_QUERY_PATH_NAME=q
endef

.PHONY: test
test: # Run all of the tests
	cargo test --workspace --target $(TARGET) $(CARGOARGS)

.PHONY: genconfig
genconfig: # Generate a configuration file for building the agent
ifeq (,$(wildcard $(CONFIG)))
	@echo -e '$(subst $(newline),\n,${DEFAULT_CONFIG})' > $(CONFIG)
	cat $(CONFIG)
endif

.PHONY: clean
clean: # Remove the built binaries
	cargo clean
	rm -vf thanatos thanatos.exe thanatos.dll thanatos.so

.PHONY: distclean
distclean: # Remove the build configuration
	rm -vf $(CONFIG)

.PHONY: all
all: thanatos thanatos.so thanatos.exe thanatos.dll # Builds all of the build targets

.PHONY: thanatos
thanatos: # Build the Linux executable version of the thanatos agent
	@$(MAKE) _binary

.PHONY: thanatos.so
thanatos.so: # Build the Linux shared object version of the thanatos agent
	@$(MAKE) _library

.PHONY: thanatos.exe
thanatos.exe: # Build the Windows executable version of the thanatos agent
	@$(MAKE) TARGET="x86_64-pc-windows-gnu" _binary

.PHONY: thanatos.dll
thanatos.dll: # Build the Windows DLL version of the thanatos agent
	@$(MAKE) TARGET="x86_64-pc-windows-gnu" _library

# Hidden build targets for abstracting the other build targets
.PHONY: _binary
_binary: $(CONFIG) $(SOURCES)
	cargo build -p thanatos_bin --target $(TARGET) $(CARGOARGS)

.PHONY: _library
_library: $(CONFIG) $(SOURCES)
	cargo build -p thanatos_lib --target $(TARGET) $(CARGOARGS)
