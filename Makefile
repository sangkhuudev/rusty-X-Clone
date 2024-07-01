# Platform-specific configurations
ifeq ($(OS), Windows_NT)
    TRUNK_CONFIG_FILE = Trunk.win.toml
    TRUNK_RELEASE_CONFIG_FILE = Trunk-release.win.toml
else
    TRUNK_CONFIG_FILE = Trunk.toml
    TRUNK_RELEASE_CONFIG_FILE = Trunk.toml
endif

# build in release mode
.PHONY: build
build:
	# build frontend
	trunk --config $(TRUNK_RELEASE_CONFIG_FILE) build
	# build backend
	cargo build --release --workspace --exclude frontend

# run cargo check
.PHONY: check
check:
	cargo check -p frontend --target wasm32-unknown-unknown
	cargo check --workspace --exclude frontend

# run cargo clippy
.PHONY: clippy
clippy:
	cargo clippy -p frontend --target wasm32-unknown-unknown
	cargo clippy --workspace --exclude frontend

# run clippy fix
.PHONY: fix
fix:
	cargo clippy -p frontend --fix --target wasm32-unknown-unknown --allow-dirty
	cargo clippy --workspace --fix --exclude frontend --allow-dirty

# build docs. use --open to open in browser
.PHONY: doc
doc:
	cargo doc -F docbuild $(ARGS)

# run frontend devserver. use --open to open a new browser
.PHONY: serve-frontend
serve-frontend:
	trunk --config $(TRUNK_CONFIG_FILE) serve $(ARGS)

# run API server
.PHONY: serve-api
serve-api:
	cargo run -p uchat_server $(ARGS)

# set up project dependencies
.PHONY: init
init:
	cargo run -p project-init
	cd frontend && npm install

# migration related commands
# apply migrations
.PHONY: db-migrate
db-migrate:
	diesel migration run
	# test migration
	diesel migration redo
	psql -d postgres -c 'DROP DATABASE uchat_test;'

# reset the database
.PHONY: db-reset
db-reset:
	diesel database reset
	psql -d postgres -c 'DROP DATABASE uchat_test;' || true

# create a new database migration
.PHONY: db-new-migration
db-new-migration:
	diesel migration generate $(NAME)
