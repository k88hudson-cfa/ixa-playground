.PHONY: clean install summary build serve new

clean:
	@echo "Cleaning all build artifacts"
	@rm -rf target/* book/*
	@cargo clean

install:
	@echo "Installing cargo dependencies"
	@cargo install mdbook

build: summary
	@echo "Building the book"
	@mdbook build

serve: summary
	@echo "Serving the book"
	@mdbook serve --watcher native

new:
	@if [ -z "$(example)" ]; then \
		echo "Usage: make new example=directory_name"; \
		exit 1; \
	fi
	@if [ -e "examples/$(example)" ]; then \
		echo "Error: examples/$(example) already exists"; \
		exit 1; \
	fi
	@echo "Creating new example: $(example)"
	@cp -r src/template "examples/$(example)"
