# Determine TRUNK_CONFIG_FILE based on OS type
ifeq ($(shell uname -s), Linux)
    TRUNK_CONFIG_FILE := Trunk.toml
    TRUNK_RELEASE_CONFIG_FILE := Trunk.toml
else
    TRUNK_CONFIG_FILE := Trunk.win.toml
    TRUNK_RELEASE_CONFIG_FILE := Trunk-release.win.toml
endif

.PHONY: build
build:
	# Build frontend using trunk
	trunk --config $(TRUNK_RELEASE_CONFIG_FILE) build
	# Build backend using cargo
	cargo build --release --workspace --exclude frontend

.PHONY: check
check:
	# Run cargo check for frontend wasm target
	cargo check -p frontend --target wasm32-unknown-unknown
	# Run cargo check for entire workspace excluding frontend
	cargo check --workspace --exclude frontend

.PHONY: clippy
clippy:
	# Run cargo clippy for frontend wasm target
	cargo clippy -p frontend --target wasm32-unknown-unknown
	# Run cargo clippy for entire workspace excluding frontend
	cargo clippy --workspace --exclude frontend

.PHONY: fix
fix:
	# Run cargo clippy fix for frontend wasm target
	cargo clippy -p frontend --fix --target wasm32-unknown-unknown
	# Run cargo clippy fix for entire workspace excluding frontend
	cargo clippy --workspace --fix --exclude frontend

.PHONY: doc
doc:
	# Build documentation using cargo doc
	cargo doc -F docbuild $(ARGS)

# .PHONY: serve-frontend
# serve-frontend:
# 	# Run frontend devserver using trunk
# 	trunk --config $(TRUNK_CONFIG_FILE) serve $(ARGS)

.PHONY: serve-tailwind
serve-tailwind:
	# Run frontend devserver using tailwind
	cd frontend && npx tailwindcss -i ./input.css -o ./assets/tailwind.css --watch
.PHONY: serve-frontend
serve-frontend:
	# Run frontend devserver using dioxus
	cd frontend && dx serve

.PHONY: serve-api
serve-api:
	# Run API server using cargo watch
	cargo watch -x 'run -p uchat_server'

.PHONY: init
init:
	# Set up project dependencies
	cargo run -p project-init
	cd frontend && npm install

.PHONY: db-migrate
db-migrate:
	# Apply migrations using diesel
	diesel migration run
	diesel migration redo
	psql -d postgres -c 'DROP DATABASE uchat_test;'

.PHONY: db-reset
db-reset:
	# Reset the database using diesel
	diesel database reset
	psql -d postgres -c 'DROP DATABASE uchat_test;' || true

.PHONY: db-new-migration
db-new-migration:
	# Create a new database migration using diesel
	diesel migration generate $(NAME)
