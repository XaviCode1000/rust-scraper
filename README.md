# 🦀 Rust Scraper

[![CI](https://github.com/XaviCode1000/rust-scraper/actions/workflows/ci.yml/badge.svg)](https://github.com/XaviCode1000/rust-scraper/actions/workflows/ci.yml)

Scraper web de alto rendimiento que utiliza Brave Browser para renderizar JavaScript y convertir contenido HTML a Markdown, optimizado para RAG (Retrieval-Augmented Generation).

## ✨ Características

- ✅ Renderizado JavaScript con Brave Browser headless (CDP)
- ✅ Conversión HTML → Markdown limpia y estructurada
- ✅ Logging estructurado con control vía `RUST_LOG`
- ✅ Respeto automático a robots.txt
- ✅ Manejo de errores con tipos personalizados
- ✅ Rate limiting configurable (250ms delay por defecto)

## 🚀 Requisitos

- [Rust](https://rustup.rs/) (1.85+ para Edition 2024)
- [Brave Browser](https://brave.com/) instalado

### Verificar instalación

```bash
rustc --version
brave --version  # o which brave
```

## 📦 Instalación

```bash
# Clonar repositorio
git clone <repo-url>
cd rust_scraper

# Compilar en modo release
cargo build --release
```

## 🎯 Uso

1. Editar `src/main.rs` para configurar la URL objetivo
2. Ejecutar:

```bash
# Modo release (producción)
cargo run --release

# Con logs detallados
RUST_LOG=debug cargo run

# Logs específicos de la app
RUST_LOG=rust_scraper=debug cargo run
```

## 📁 Estructura

```
├── src/
│   ├── main.rs      # Orquestación y validación
│   ├── config.rs    # Configuración de Brave y logging
│   ├── scraper.rs   # Lógica de web scraping
│   └── markdown.rs  # Conversión HTML → Markdown
├── Cargo.toml
├── README.md
└── LICENSE
```

## 📄 Licencia

MIT License - ver [LICENSE](LICENSE) para más detalles.
