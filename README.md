# 🕷️ Rust Scraper

**Web scraper profesional con interfaz interactiva, limpieza semántica con IA y exportación a Obsidian.**

---

## 🚀 ¿Qué hace?

Rust Scraper es una herramienta CLI para extraer contenido de sitios web con estas características:

- **Modo interactivo (TUI)** — Explora URLs, selecciona qué descargar y confirma antes de procesar
- **Modo headless** — Automatización completa para scripts y pipelines
- **Limpieza semántica con IA** — Extrae contenido relevante automáticamente (opcional)
- **Exportación a Obsidian** — Guarda notas directamente en tu vault con metadatos
- **Soporte de sitemaps** — Descubre automáticamente todas las páginas de un sitio
- **Descarga de assets** — Imágenes y documentos con detección automática
- **Formatos de exportación** — Markdown, JSONL, JSON, y Vector (para RAG)

---

## 📦 Instalación

### Desde código fuente

```bash
git clone https://github.com/XaviCode1000/rust-scraper.git
cd rust-scraper
cargo build --release
```

El binario se encuentra en `target/release/rust_scraper`.

### Requisitos

- **Rust:** 1.88+
- **Cargo:** 1.88+

### Características opcionales

| Característica | Flag de compilación | Descripción |
|---------------|-------------------|-------------|
| Limpieza con IA | `--features ai` | Extracción semántica con modelos locales |
| Descarga de imágenes | `--features images` | Detecta y descarga imágenes |
| Descarga de documentos | `--features documents` | Detecta y descarga PDFs, docs, etc. |
| Todo (excepto IA) | `--features full` | Habilita images + documents |

**Ejemplo con IA:**
```bash
cargo build --release --features ai
```

---

## 🎯 Uso rápido

### Modo interactivo (recomendado para empezar)

Explora URLs, selecciona qué descargar y confirma antes de procesar:

```bash
./target/release/rust_scraper --url https://example.com --interactive
```

#### Controles del TUI

| Tecla | Acción |
|-------|--------|
| `↑` / `↓` | Navegar URLs |
| `Espacio` | Seleccionar/deseleccionar |
| `A` | Seleccionar todo |
| `D` | Deseleccionar todo |
| `Enter` | Confirmar descarga |
| `Y` / `N` | Confirmación final |
| `q` | Salir |

### Modo headless (para scripts)

```bash
# Scrapear todas las URLs de un sitio
./target/release/rust_scraper --url https://example.com

# Con sitemap (descubre automáticamente desde robots.txt)
./target/release/rust_scraper --url https://example.com --use-sitemap

# Exportar a Markdown
./target/release/rust_scraper --url https://example.com --format markdown --output ./output
```

### Limpieza semántica con IA

```bash
# Habilitar limpieza con IA
./target/release/rust_scraper \
  --url https://example.com \
  --clean-ai \
  --export-format jsonl

# Ajustar umbral de similitud (0.0 - 1.0)
./target/release/rust_scraper \
  --url https://example.com \
  --clean-ai \
  --ai-threshold 0.3
```

### Exportación a Obsidian

```bash
# Guardar directamente en tu vault (quick-save)
./target/release/rust_scraper \
  --url https://example.com/article \
  --obsidian-wiki-links \
  --obsidian-rich-metadata \
  --quick-save

# Con vault explícito
./target/release/rust_scraper \
  --url https://example.com/article \
  --vault ~/Obsidian/MyVault \
  --obsidian-wiki-links \
  --obsidian-tags "rust,web,scraping" \
  --obsidian-relative-assets
```

**Quick-save:**
- Detecta el vault automáticamente (CLI > variable de entorno > config > escaneo)
- Guarda en `{vault}/_inbox/YYYY-MM-DD-slug.md`
- Abre la nota en Obsidian si está ejecutándose (Linux)

---

## 📚 Formatos de exportación

| Formato | Descripción | Uso típico |
|---------|-------------|-------------|
| `markdown` | Markdown con formato | Lectura humana, blogs |
| `jsonl` | JSON Lines (un objeto por línea) | RAG, procesamiento por lotes |
| `json` | JSON completo | APIs, integraciones |
| `vector` | JSON con embeddings | Bases de datos vectoriales |

### Ejemplo de uso con RAG

```bash
# Exportar en formato JSONL para RAG
./target/release/rust_scraper \
  --url https://example.com \
  --clean-ai \
  --export-format jsonl \
  --output ./rag-data
```

---

## ⚙️ Opciones avanzadas

```bash
# Ejemplo completo con todas las opciones
./target/release/rust_scraper \
  --url https://example.com \
  --output ./output \
  --format markdown \
  --download-images \
  --download-documents \
  --use-sitemap \
  --concurrency 5 \
  --delay-ms 1000 \
  --max-pages 100 \
  --verbose

# Concurrencia automática (detecta CPU)
./target/release/rust_scraper \
  --url https://example.com \
  --concurrency auto
```

### Modo silencioso (para scripts)

```bash
# Sin barras de progreso ni output decorativo
./target/release/rust_scraper \
  --url https://example.com \
  --quiet
```

### Dry-run (previsualizar sin descargar)

```bash
# Ver qué URLs se descargarían sin procesar
./target/release/rust_scraper \
  --url https://example.com \
  --dry-run
```

---

## 🔧 Configuración

Crea un archivo de configuración en `~/.config/rust-scraper/config.toml`:

```toml
# Configuración por defecto
vault = "~/Obsidian/MyVault"
concurrency = 4
delay_ms = 1000
max_pages = 50
output = "./output"
format = "markdown"
```

Las opciones de línea de comandos tienen prioridad sobre la configuración.

---

## 📖 Documentación adicional

- **[Guía de desarrollo](DEVELOPMENT.md)** — Para contribuidores
- **[Documentación técnica](docs/)** — Arquitectura, integración con Obsidian, limpieza con IA
- **[CHANGELOG](CHANGELOG.md)** — Historial de cambios

---

## 🤝 Contribuir

Las contribuciones son bienvenidas. Por favor lee [CONTRIBUTING.md](docs/CONTRIBUTING.md) antes de enviar un PR.

---

## 📄 Licencia

MIT OR Apache-2.0
