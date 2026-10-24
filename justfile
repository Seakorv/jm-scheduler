# Show all recipes
default:
    @just --list

# Format all code
fmt: fmt-backend fmt-frontend

# Check formatting without changing files (used by CI)
fmt-check: fmt-check-backend fmt-check-frontend

# Lint + typecheck everything
lint: lint-backend lint-frontend

# Run all tests
test: test-backend test-frontend

# Everything CI runs
check: fmt-check lint test

# Format, then lint and test (run before committing)
ready: fmt lint test

[working-directory: 'backend']
fmt-backend:
    cargo fmt

[working-directory: 'frontend']
fmt-frontend:
    pnpm exec prettier --write .

[working-directory: 'backend']
fmt-check-backend:
    cargo fmt --check

[working-directory: 'frontend']
fmt-check-frontend:
    pnpm exec prettier --check .

[working-directory: 'backend']
lint-backend:
    cargo clippy --all-targets -- -D warnings

[working-directory: 'frontend']
lint-frontend:
    pnpm lint
    pnpm exec tsc -b

[working-directory: 'backend']
test-backend:
    cargo test

[working-directory: 'frontend']
test-frontend:
    pnpm test
