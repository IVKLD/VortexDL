# Run all formatters
fmt: fmt-backend fmt-frontend

# Run all linters
lint: lint-backend lint-frontend

# Format backend code
fmt-backend:
    cargo fmt --all

# Lint backend code
lint-backend:
    cargo clippy -- -D warnings

# Run backend in watch mode
backend:
    cargo watch -w src -w Cargo.toml -i "frontend/*" -x "run -- --serve"

# Format frontend code
fmt-frontend:
    cd frontend && yarn lint --fix

# Lint frontend code
lint-frontend:
    cd frontend && yarn lint

# Run frontend development server
frontend:
    cd frontend && yarn start

# Install all dependencies
install:
    cargo fetch
    cd frontend && yarn install

# Build backend binary only
build:
    cargo build --release

# Build full distribution (backend + frontend)
dist:
    cargo build --release --features web
