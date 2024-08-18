post-build: build
	rm ../forwarder/ib-analyzer
	cp ./target/debug/server-tower ../forwarder/ib-analyzer

build:
	cargo build

.PHONY: build post-build