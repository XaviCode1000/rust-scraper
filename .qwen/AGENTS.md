# 🦀 Rust Expert - Qwen Code Agent System

Sistema de agentes especializados para desarrollo Rust con Qwen Code. Incluye **1 agente orquestador** y **9 subagentes expertos**, cada uno con sus propios skills y permisos configurados.

---

## 📁 Estructura

```
.qwen/
├── settings.json              # Configuración principal
├── agents/
│   ├── rust-orquestrator.md   # Agente primario coordinador
│   ├── rust-researcher.md     # Investigación (2 intentos fallidos)
│   ├── rust-reviewer.md       # Code review y anti-patterns
│   ├── rust-tester.md         # Testing y benchmarks
│   ├── rust-docs.md           # Documentación
│   ├── rust-async.md          # Async y Tokio
│   ├── rust-memory.md         # Memoria y ownership
│   ├── rust-performance.md    # Optimización y profiling
│   ├── rust-errors.md         # Error handling
│   ├── rust-types.md          # Type system
│   └── rust-project.md        # Estructura de proyectos
├── skills/
│   └── rust-skills/           # 179 reglas rust-skills (symlink)
└── AGENTS.md                  # Este archivo
```

---

## 🚀 Instalación

### Configuración por Proyecto

La configuración `.qwen/` ya está en este proyecto. Los agentes están disponibles automáticamente cuando usás Qwen Code en este directorio.

### Configuración Global (Opcional)

Para usar los agentes en todos tus proyectos:

```bash
# Copiar agentes a configuración global
cp -r /home/gazadev/Dev/my_apps/rust_scraper/.qwen/agents ~/.qwen/agents/

# Copiar skills
cp -r /home/gazadev/Dev/my_apps/rust_scraper/.qwen/skills ~/.qwen/skills/
```

---

## 👥 Equipo de Agentes

### Agente Primario

| Agente | Rol | Descripción |
|--------|-----|-------------|
| `rust-orquestrator` | **Coordinador** | Orquesta los 9 subagentes especializados. Delega tareas según especialidad. |

### Subagentes Especializados

| Agente | Especialidad | Skills | Cuándo Usar |
|--------|-------------|--------|-------------|
| `rust-researcher` | 🔍 Investigación | Web search, Context7 MCP, docs oficiales | **Automático**: 2 intentos fallidos de cualquier subagente |
| `rust-reviewer` | 🧐 Code Review | anti-*, api-*, lint-*, name-* | Review de PRs, detectar anti-patterns |
| `rust-tester` | 🧪 Testing | test-*, perf-* | Escribir tests, mocks, benchmarks |
| `rust-docs` | 📚 Documentación | doc-*, name-* | Documentar APIs, README, ejemplos |
| `rust-async` | ⚡ Async | async-*, own-mutex/rwlock/arc | Código Tokio, channels, concurrency |
| `rust-memory` | 💾 Memoria | mem-*, own-* | Optimizar allocaciones, borrowing |
| `rust-performance` | 🚀 Performance | opt-*, perf-* | Profiling, LTO, hot paths |
| `rust-errors` | ⚠️ Errores | err-* | thiserror, anyhow, Result |
| `rust-types` | 🏷️ Types | type-* | Newtypes, enums, generics |
| `rust-project` | 📂 Proyecto | proj-*, mod-* | Workspaces, módulos, visibilidad |

---

## 🎯 Protocolo de 2 Intentos Fallidos → rust-researcher

**CARACTERÍSTICA CRÍTICA:** Todos los subagentes están configurados para invocar **automáticamente** a `rust-researcher` cuando:

1. **Primer intento:** Implementa algo → no funciona / error de compilación
2. **Segundo intento:** Corrige → sigue sin funcionar
3. **Tercer paso:** **AUTOMÁTICAMENTE** invoca `rust-researcher` ANTES de seguir

```markdown
task({
    agent: "rust-researcher",
    prompt: "Intenté implementar [X] 2 veces y falla.

    Error 1: [mensaje]
    Error 2: [mensaje]

    Investigá en:
    1. Documentación oficial (2026)
    2. Código real en GitHub (tokio, serde, axum)
    3. Context7 MCP para crates específicos"
})
```

Esto evita que el equipo pierda tiempo intentando soluciones incorrectas.

---

## 🔐 Permisos y Control

### Permisos Globales (settings.json)

```json
{
  "permissions": {
    "defaultMode": "default",
    "confirmShellCommands": true,
    "confirmFileEdits": true,
    "allowlist": {
      "shell": [
        "cargo *",
        "cargo build*",
        "cargo test*",
        "cargo clippy*",
        "cargo fmt*",
        "rustfmt *",
        "rg *",
        "fd *",
        "eza *",
        "bat *"
      ]
    }
  }
}
```

### Control de Usuario

| Permiso | Configuración | Qué Significa |
|---------|--------------|---------------|
| `defaultMode` | `default` | Todas las ediciones y comandos requieren aprobación |
| `confirmShellCommands` | `true` | Comandos fuera de allowlist requieren aprobación |
| `confirmFileEdits` | `true` | **Todas las ediciones requieren aprobación del usuario** |
| `allowlist.shell` | cargo *, rustfmt *, etc. | Comandos de build/test automáticos |

### Approval Modes

Qwen Code soporta 4 modos de aprobación:

| Modo | Ediciones | Comandos | Cuándo Usar |
|------|-----------|----------|-------------|
| **Plan** | ❌ Read-only | ❌ No ejecuta | Exploración, planning |
| **Default** | ✅ Pregunta | ✅ Pregunta | Desarrollo normal (recomendado) |
| **Auto-Edit** | ✅ Auto | ❌ Pregunta | Refactoring, cambios seguros |
| **YOLO** | ✅ Auto | ✅ Auto | Solo en proyectos personales de confianza |

Cambiar modo: `/approval-mode <mode>` o `Shift+Tab` para ciclar.

---

## 💬 Uso

### Invocar Orquestrador

El orquestador se usa automáticamente cuando pedís tareas complejas. También podés invocarlo explícitamente:

```
@rust-orquestrator necesito implementar una API async con tests
```

### Invocar Subagente Directamente

```
@rust-reviewer revisá este módulo en busca de anti-patterns

@tester escribí tests unitarios para este código

@rust-async revisá si hay lock across await en este código
```

### Delegación Automática

El `rust-orquestrator` delega automáticamente según la tarea:

```
Usuario: "Necesito implementar una API async con tests"

rust-orquestrator → rust-async: "Implementá la API async"
rust-orquestrator → rust-tester: "Escribí tests para la API"
rust-orquestrator → rust-reviewer: "Revisá anti-patterns"
rust-orquestrator → rust-docs: "Documentá la API pública"
```

---

## 📋 179 Skills Disponibles

Los 179 skills de rust-skills están organizados por categoría y asignados a los agentes correspondientes:

| Categoría | Count | Agente Principal |
|-----------|-------|-----------------|
| `anti-*` | 15 | rust-reviewer |
| `api-*` | 15 | rust-reviewer |
| `async-*` | 15 | rust-async |
| `doc-*` | 22 | rust-docs |
| `err-*` | 12 | rust-errors |
| `lint-*` | 11 | rust-reviewer |
| `mem-*` | 15 | rust-memory |
| `name-*` | 16 | rust-reviewer, rust-docs |
| `opt-*` | 12 | rust-performance |
| `own-*` | 12 | rust-memory, rust-async |
| `perf-*` | 11 | rust-tester, rust-performance |
| `proj-*` | 11 | rust-project |
| `test-*` | 13 | rust-tester |
| `type-*` | 10 | rust-types |
| `mod-*` | 2 | rust-project |

---

## 🔧 Configuración de Herramientas Externas

### LSP

rust-analyzer está configurado automáticamente en `settings.json`:

```json
{
  "lsp": {
    "rust": {
      "env": {
        "RUST_LOG": "info"
      }
    }
  }
}
```

### MCP Servers

Tenés 4 MCP servers configurados en `settings.json`:

| Server | Tipo | Qué provee |
|--------|------|------------|
| `context7` | HTTP | Documentación de crates (docs.rs) |
| `exa` | HTTP | Búsqueda web, código, research profundo |
| `jina` | HTTP | Web fetch, search, AI services |
| `engram` | Local | Memoria persistente entre sesiones |

```json
{
  "mcpServers": {
    "context7": {
      "httpUrl": "https://mcp.context7.com/mcp"
    },
    "exa": {
      "httpUrl": "https://mcp.exa.ai/mcp?exaApiKey=..."
    },
    "jina": {
      "httpUrl": "https://mcp.jina.ai/v1",
      "headers": {
        "Authorization": "Bearer ..."
      }
    },
    "engram": {
      "command": "engram",
      "args": ["mcp", "--tools=agent"]
    }
  }
}
```

### Model Configuration

```json
{
  "model": {
    "name": "opencode/minimax-m2.5-free",
    "temperature": 0.3
  }
}
```

---

## 📝 Ejemplos de Uso

### Code Review

```
@rust-reviewer revisá este PR en busca de:
- anti-clone-excessive
- anti-unwrap-abuse
- anti-lock-across-await

Focus en CRITICAL primero.
```

### Nueva Feature Async

```
@rust-orquestrator necesito implementar una API async con:
- Tokio channels bounded
- Cancellation con CancellationToken
- Tests unitarios
- Documentación completa

Coordiná el equipo.
```

### Debugging de Borrow Checker

```
@rust-memory el borrow checker no me deja compilar esto.
Intenté 2 veces y sigo teniendo errores.

[código]

¿Podés revisar el ownership?
```

### Optimización de Performance

```
@rust-performance profileá este hot path y sugerí optimizaciones.

[código + benchmark actual]

Focus en:
- LTO y codegen-units
- Inline estratégico
- Cache-friendly layouts
```

---

## 🎨 Personalidad de los Agentes

Todos los agentes comparten la personalidad **RUST-JARVIS**:

- **Directos y confrontacionales** - Sin filtro, autoridad técnica
- **Rioplatense** - boludo, quilombo, dejate de joder, está piola
- **Frustrados con mediocridad** - tutorial programmers, shortcuts, unwrap() en prod
- **"Sí, señor"** - Confirmaciones clave
- **Push back** - Si pedís código sin contexto, te dicen "bancá, primero entendamos los conceptos"

---

## 🔧 Troubleshooting

### Los agentes no aparecen

Verificá que los archivos estén en la ubicación correcta:

```bash
# Project agents
ls .qwen/agents/rust-*.md

# Skills
ls .qwen/skills/rust-skills/
```

### Permisos bloquean acciones

Revisá `settings.json` y ajustá los permisos según necesites. Por defecto:
- `defaultMode: default` - Todas las ediciones requieren aprobación
- `confirmShellCommands: true` - Comandos fuera de allowlist requieren aprobación

### Cambiar a Auto-Edit Mode

Para desarrollo más rápido en tu proyecto personal:

```
/approval-mode auto-edit
```

Esto permite ediciones automáticas (los comandos shell siguen requiriendo aprobación).

---

## 📚 Recursos

- [Qwen Code Docs - Agents](https://qwen.ai/docs/agents)
- [Qwen Code Docs - Skills](https://qwen.ai/docs/skills)
- [Qwen Code Docs - Approval Modes](https://qwen.ai/docs/approval-mode)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [Rust Performance Book](https://nnethercote.github.io/perf-book/)

---

## 🤝 Contribución

Los 179 skills originales están en `/home/gazadev/Dev/my_apps/rust_scraper/.opencode/skills/rust-skills/` (symlink en `.qwen/skills/rust-skills/`).

Para agregar nuevos agentes:

1. Creá `agents/nuevo-agente.md` con frontmatter YAML
2. Definí `tools` que puede usar
3. Actualizá este AGENTS.md

---

**Versión:** 1.0.0 (Qwen Code)
**Autor:** Rust Expert Team
**License:** MIT
**Basado en:** OpenCode Rust Expert System
