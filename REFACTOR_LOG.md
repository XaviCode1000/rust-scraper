
## Fase 5: Ejecución de Acciones Aprobadas

### Acción 1: ✅ Eliminar 10 dependencias unused
- **Removed**: flate2, md5, memmap2, ndarray, ort, pulldown-cmark-to-cmark, slug, tokio-util, tracing-appender, urlencoding
- **Verified**: `cargo machete` → 0 unused deps
- **Commit**: `chore(deps): remove 10 unused dependencies`

### Acción 2a: ✅ Extraer wikilinks.rs de obsidian.rs
- **obsidian.rs**: 678 → 190 lines (-72%)
- **wikilinks.rs**: 428 lines (new)
- **Tests**: 16 wikilinks + 5 obsidian = 21/21 passing
- **Commit**: `refactor(obsidian): extract wikilinks module from obsidian converter`

### Acción 2b: ⏭️ SKIP - crawler deprecated.rs
- Only 2 deprecated functions (~105 lines total) — not worth separate module

### Acción 2c: ✅ Extraer sitemap_config.rs
- **sitemap_parser.rs**: 753 → 581 lines (-23%)
- **sitemap_config.rs**: 148 lines (new)
- **Tests**: 21/21 passing
- **Commit**: `refactor(sitemap): extract config from sitemap parser`

### Acción 2d: ✅ Extraer cache_config.rs
- **model_cache.rs**: 649 → 254 lines (-61%)
- **cache_config.rs**: 137 lines (new)
- **Tests**: compile clean, ai feature-gated tests timeout on hardware
- **Commit**: `refactor(ai): extract cache config from model cache`

### Acción 2e: ⏭️ SKIP - http_client retry.rs
- Only 33 lines of retry-related code — not worth separate module

### Acción 3: ⚠️ BLOCKED — time crate RUSTSEC-2026-0009
- **Blocker**: `tract-linalg v0.21.15` requires `time >=0.3.23, <0.3.42`
- **Required**: `time >=0.3.47` for RUSTSEC fix
- **Resolution**: Wait for tract-onnx upstream to update time constraint
- **Status**: Documented as pending

### Summary
| Action | Status | Lines Changed |
|--------|--------|---------------|
| Remove 10 unused deps | ✅ | Cargo.toml -137 lines |
| Extract wikilinks | ✅ | -54 net (678→190, +428) |
| Extract sitemap config | ✅ | -12 net (753→581, +148) |
| Extract cache config | ✅ | -258 net (649→254, +137) |
| Extract crawler deprecated | ⏭️ SKIP | — |
| Extract http retry | ⏭️ SKIP | — |
| Fix time RUSTSEC | ⚠️ BLOCKED | upstream constraint |

