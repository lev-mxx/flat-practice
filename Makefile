
SHELL = /bin/bash

build: dependencies
	source make/set_paths.bash && cargo build

test: dependencies
	source make/set_paths.bash && (cd lib/graphblas && cargo test) && (cd lib && cargo test) && cargo test

dependencies: build/rust build/graphblas
	pip3 install pyformlang --user

build/rust:
	source make/get_rust.bash && touch $@

build/graphblas:
	source make/get_graphblas.bash && touch $@

clean:
	rm -rf build
