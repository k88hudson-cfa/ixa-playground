.PHONY: clean install build serve

clean:
	@echo "Cleaning all build artifacts"
	@rm -rf target/* book/*
	@cargo clean

install:
	@echo "Installing cargo dependencies"
	@command -v mdbook >/dev/null 2>&1 || { \
		echo "mdbook not found, installing..."; \
		cargo install mdbook --force; \
	}

build: clean
	@echo "Building the book"
	@mdbook build

serve:
	@echo "Serving the book"
	@mdbook serve --watcher native
