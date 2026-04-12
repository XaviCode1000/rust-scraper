# justfile — rust_scraper
# Complementa a bacon (inner loop). Esto es para tareas manuales (outer loop).

# -- Verificación --

default: check

check:
    cargo fmt --check
    cargo clippy --all-targets --all-features -- -D warnings -W clippy::pedantic

check-fast:
    cargo check

# -- Tests --

test:
    cargo nextest run --test-threads 2

test-ai:
    cargo nextest run --test-threads 2 --features ai

# -- Auditoría --

audit:
    cargo audit
    cargo deny check
    cargo machete

# -- Coverage --

cov:
    cargo llvm-cov --html --output-dir coverage-llvm

# -- Format --

fmt:
    cargo fmt

# -- Build --

build-release:
    cargo build --release

# -- CI --

test-ci:
    cargo nextest run --profile ci

# -- Maintenance --

fix-typos:
    typos -w

# -- Setup --

setup:
    @echo "Verificando herramientas..."
    @which cargo-nextest || (echo "Falta: cargo binstall cargo-nextest"; exit 1)
    @which just || (echo "Falta: cargo binstall just"; exit 1)
    @which cargo-machete || (echo "Falta: cargo binstall cargo-machete"; exit 1)
    @which cargo-audit || (echo "Falta: cargo binstall cargo-audit"; exit 1)
    @which cargo-deny || (echo "Falta: cargo binstall cargo-deny"; exit 1)
    @which typos || (echo "Falta: cargo binstall typos-cli"; exit 1)
    @which sccache || (echo "Falta: sccache"; exit 1)
    @which mold || (echo "Falta: mold"; exit 1)
    @echo "Setup completo — todas las herramientas verificadas"
