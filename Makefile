.PHONY: clean install build serve

clean:
	@echo "Cleaning all build artifacts"
	@rm -rf target/* book/*
	@cargo clean

install:
	@echo "Installing cargo dependencies"
	@cargo install mdbook --force

build: clean
	@echo "Building the book"
	@mdbook build

serve:
	@echo "Serving the book"
	@mdbook serve --watcher native
