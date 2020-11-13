
SHELL = /bin/bash

build: dependencies
	source make/set_paths.bash && cargo build

test: dependencies
	source make/set_paths.bash && (cd lib && cargo test)

dependencies: build/rust build/graphblas
	pip3 install pyformlang --user -qq

build/rust:
	source make/get_rust.bash && touch $@

build/graphblas:
	source make/get_graphblas.bash && touch $@

clean:
	rm -rf build && (cargo clean) && (cd lib && cargo clean && (cd graphblas && cargo clean))
