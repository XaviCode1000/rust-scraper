# Playbook CI / Tests / PRs — aprendido de 7 repos Rust de élite

> Fecha: 2026-09-08. Motivación: auditoría interna — 2744 fns de test, 19 jobs en
> `ci.yml`, ~33 min de wall-clock para un merge verde, fricción de merge alta
> (`strict: true` sin merge queue). Este doc es la guía de adaptación: qué copiar,
> qué adaptar y qué NO copiar de cada repo.
>
> Cómo se obtuvo cada dato: `gh api repos/<repo>/contents/.github/workflows/<file>`
> (YAML crudo) + `gh run list` (wall-clocks reales) + `raw.githubusercontent.com`
> (CONTRIBUTING, PR templates, scripts CI). Red caótica ese día: `api.github.com`
> intermitente, DeepWiki MCP y JINA API caídos — se trabajó con `gh` + raw.

## 0. Baseline nuestro (el "antes")

| Métrica | Valor (2026-09-08) |
| ------- | ------------------ |
| Wall-clock PR verde (main, run 34274635094) | **~33 min (1999 s)** |
| Jobs en `ci.yml` | 19 |
| Job más caro | Coverage **1610 s** (~27 min) |
| 2.º más caro | Tests AI integration **1163 s** (~19 min, **required**) |
| Required checks (branch protection) | `Validate PR metadata`, `cargo-mutants (PR diff)`, `CI Gate`, `Tests (AI integration)` |
| `strict` (exige estar al día con main) | **true** — cada merge a main invalida los PRs abiertos y re-dispara 33 min |
| Merge queue | **No existe** (`auto_merge: true` suelto) |
| Tests | ~2744 fns (`#[test]`/`#[tokio::test]`/`#[rstest]` en `crates/`+`tests/`) |
| Mutants en PR | **hard gate** (`mutants-pr`, required) |

## 1. Wall-clocks comparados (runs verdes reales, `gh run list`)

| Repo | PR (push/PR event) | Cola/main | Jobs CI ppal | Nota |
| ---- | ------------------- | --------- | ------------ | ---- |
| wasmtime | **8–9 min** | 25–34 min (merge queue) | 35 | FAST_MATRIX en PR, FULL en queue |
| rust-analyzer | **7–8 min** (red rápido: 2 min) | 11–13 min (queue) | 12 | `conclusion` agregador |
| ripgrep | **9–10 min** | 10–11 min (schedule) | 5 | matriz 18 configs en ubuntu+cross |
| ruff | 11–16 min | 11 min (main) | 31 | nextest + `determine_changes` |
| typst | 15–17 min | 4–5 min* | 7 | *push a rama; matriz 3 plataformas |
| tokio | 20–28 min (red rápido: 2 min) | — | 46 | gate `basics` |
| bevy | ~24 min | 21–29 min (queue) | 18 | 595 PRs abiertos, merge queue |
| **webfang** | **~33 min** | — (sin queue) | 19 | required incluye mutants + AI 19 min |

Conclusión: **nadie** pone mutants ni integration de 19 min como required en PR.
Todos separan *gate rápido de PR* de *verificación pesada en queue/main/schedule*.

## 2. Patrones universales (los 7 coinciden — copiar sin dudar)

1. **Un solo check agregador como required.** wasmtime (`ci-status`, `if: always()`),
   ruff (`required-checks-passed`, acepta `skipped`), rust-analyzer (`conclusion`,
   con comentario explícito de la trampa `skipped == success`), typst (`tests`,
   citando el truco de branch protection). Nosotros ya tenemos `CI Gate` — bien,
   pero debe agregar un SET PEQUEÑO, no todo.
2. **Fuzz = solo compila en PR.** ripgrep (`cargo check` en `fuzz/`), typst
   (`cargo fuzz build --dev`), tokio (`check-fuzzing`). Correr fuzz en PR no lo
   hace nadie.
3. **Miri acotado, nunca la suite entera en PR.** typst: UN target
   (`cargo miri test -p typst-library test_miri`). tokio: splits lib/test/doc.
   wasmtime: opt-in por token `prtest:miri`.
4. **`cancel-in-progress` en PRs** (todos). Ya lo tenemos.
5. **Pinned action SHAs + `rust-cache`/`Swatinem` con `save-if: main-only`**
   (ruff). Abarata y estabiliza.
6. **Los checks caros informan, no bloquean.** ruff: fuzz, ecosystem, benches,
   codspeed fuera del required. bevy: miri y validaciones fuera del camino
   crítico.

## 3. Técnicas por repo (qué → evidencia → adaptación → esfuerzo)

### 3.1 wasmtime — tiering total (PRIORIDAD 1 para nosotros)

- **FAST_MATRIX vs FULL_MATRIX** (`ci/build-test-matrix.js`, 440 líneas):
  PR corre **1 job** (Linux x86_64); la merge queue y main corren la matriz
  completa. Resultado medido: PR 8–9 min, queue 25–34 min.
- **Opt-in por commit-token**: `prtest:full`, `prtest:miri`, `prtest:capi`,
  `prtest:debug`, `prtest:platform-checks` en el mensaje del commit activan
  suites pesadas a demanda. Auto-trigger por path (`fuzz/` → nightly,
  `Cargo.lock` → audit, `gc` → gc-zeal, `cranelift/codegen/src/isa/<x>` →
  config de esa ISA).
- **Sharding por crates**: 3 buckets genéricos (round-robin sobre
  `cargo metadata`) + buckets singleton para los crates lentos + sub-buckets
  por `--test` para el crate dominante. Build de deps duplicado, ejecución
  paralelizada (compensa en targets lentos).
- **Triage horario** (`triage.yml`): labeler por paths + `subscribe-to-label`
  (@-menciona suscriptores) + `label-messager` (respuestas enlatadas).
- **`file-issue-on-error`**: un build pesado (dispatch/schedule) que falla
  **abre un issue automáticamente**. Los fallos del tier pesado nunca se
  pierden en silencio.
- **Merge queue nativa** (`merge_group` en `main.yml`; colas visibles como
  `gh-readonly-queue/...` en `gh run list`).
- Adaptación: `scripts/build-test-matrix.{js,py}` propio con
  `FAST = [unit+clippy+fmt]` / `FULL = todo`; tokens `citest:full|miri|ai`;
  mover coverage+AI-integration+mutants-weekly al tier queue/main/schedule.
  Esfuerzo: **M**.

### 3.2 ruff — miles de tests sin dolor (PRIORIDAD 1)

- **`determine_changes`** (`ci.yaml`): merge-base real (`git fetch --unshallow`
  + `git diff MERGE_BASE...HEAD -- <pathspecs>`) con flags por área
  (parser/linter/formatter/code/fuzz/ty/playground/benchmarks/release). Cada
  job pesado lleva `if: <flag> || main`. Los docs-only no compilan nada.
- **Escape hatch por label**: `no-test` / `no-build` en el PR salta jobs.
  (Disciplina: solo el maintainer la pone.)
- **nextest perfil `ci`** (`.config/nextest.toml`): `slow-timeout 1s`
  (marca lentos), `terminate-after 60s` (guillotina deadlocks),
  `fail-fast = false`, resumen final de lentos. Recomendado oficialmente en su
  CONTRIBUTING (`cargo nextest run`).
- **insta anti-sprawl**: `cargo insta test --unreferenced reject` — un snapshot
  huérfano **falla el CI**. Nuestro `.snap.new` gitignored + review manual es
  compatible; nos falta el reject automático.
- **Dogfooding como tier**: `ty` se chequea a sí mismo sobre su propio repo en
  CI. Equivalente nuestro: `webfang` scrapeando fixtures propias versionadas.
- **codspeed**: benches con gate de regresión (instrumented + walltime).
  Nuestro `benches.yml` existe — evaluar si gatea o solo informa.
- **Runners grandes + mold + `line-tables-only`**: `depot-ubuntu-22.04-16`,
  `setup-mold`, `CARGO_PROFILE_DEV_DEBUG=line-tables-only` (backtraces sin el
  coste de debuginfo full). Ya tenemos `DEBUG=0`+no-incremental en mutants;
  generalizar.
- **Required mínimo**: 11 jobs (`fmt, clippy, hawk, test-linux, test-wasm,
  msrv, shellcheck, scripts, prek, docs, python-package`). Todo lo demás
  informa.
- Adaptación: `determine_changes` con áreas `core/ai/mcp/cli/docs/ci`;
  perfil nextest `ci` propio; `--unreferenced reject`; required = agregador
  sobre ~6 jobs. Esfuerzo: **S–M**.

### 3.3 tokio — matriz de features sin explosión (PRIORIDAD 2)

- **Gate `basics`**: agregador tonto (`needs: [clippy, fmt, docs, minrust]`,
  `run: exit 0`) del que **todo** cuelga (`needs: basics`). Rojo en 2 min
  medidos (run 34258764834). El fail-fast más barato posible.
- **loom fuera del CI de PR**: workflow `loom.yml` separado; en PR solo corre
  con label `R-loom-blocking`, siempre en push a master. Patrón generalizable:
  *herramienta pesada = workflow separado + label opt-in + siempre en main*.
- **MSRV como política, no como matriz**: documentado (≥6 meses, sube solo en
  minor) + un job `minrust`. Nada de combinatoria.
- Extras copiables: `check-spelling` (typos), `semver-checks`, `check-readme`
  (docs sincronizados), LTS documentado.
- Adaptación: nuestro `feature-matrix` (265 s) pasa a: job `basics` requerido
  + matriz completa solo en queue/main; loom-style para nuestros tests
  de concurrencia si aparecen. Esfuerzo: **S**.

### 3.4 typst — cultura snapshot lean (PRIORIDAD 2)

- **Un required**: job `tests` agregador (truco
  `orgs/community/discussions/4324`). Un solo nombre en branch protection.
- **Harness `testit`**: `suite/` (inputs `.typ`, N tests por archivo vía
  `--- nombre attrs ---`) + `ref/` (referencias) + **`hashed references`**
  (32 bytes por test, anti-bloat) + `store/` (output vivo, gitignored, con
  auto-regen desde el commit que fijó el hash). Reporte HTML con diffs al
  fallar. Flags `--stages eval,paged,html` (velocidad en dev) y `--update`
  (bless explícito).
- **Política "assertions > reference images"** + tope 20 KiB por imagen ref
  (atributo `large` como excepción vergonzante).
- **`checks` anti-enmascaramiento**: clippy por crate con
  `--no-default-features` (la unificación de features oculta errores) +
  `git diff --exit-code` (docs generados frescos).
- **Cultura PR** (CONTRIBUTING): PRs pequeños, discutir diseño ANTES en
  issue/Discord, CI verde antes de la primera review, **cerrar PRs
  estancados** esperando al contribuidor, política no-AI explícita.
- Adaptación: pensar nuestro corpus behavioral como `suite/` con stages
  (fast: parse-only vs full: network-mock); tope de tamaño a snapshots;
  `git diff --exit-code` para generados; política de cierre de PRs propios
  viejos. Esfuerzo: **M** (harness) / **S** (políticas).

### 3.5 ripgrep — eficiencia single-maintainer (PRIORIDAD 1 en espíritu)

- **Matriz 18 configs, 9–10 min**: todo en `ubuntu-latest` con binario `cross`
  **pineado** (compilar cross cuesta; el binario no). Solo mac/win nativos
  puntuales. Comentarios de coste explícitos ("esto casi duplica el runtime,
  se salta en cross").
- **`rgtest!(r<issue>)`**: cada regression test lleva el número de issue en el
  nombre (`tests/regression.rs`, 1744 líneas). Trazabilidad total sin
  herramienta extra.
- **5 jobs, 271 líneas**: test, wasm, rustfmt, docs, fuzz(build-only).
  Permisos mínimos documentados en el propio YAML.
- Adaptación: fijar `cross` si ampliamos targets; nombrar regression tests
  `r<issue>`; podar `ci.yml` con tijera de "¿qué confianza aporta este job por
  minuto?". Esfuerzo: **S**.

### 3.6 bevy — escala de PRs (copiar selectivo, PRIORIDAD 3)

- **Merge queue nativa** + taxonomía de **60 labels**
  (`C-*` categoría, `A-*` área, `P-*` prioridad, `O-*` OS, `S-*` estado,
  `D-*` dificultad, `M-*`, `X-Needs-SME`). `S-Ready-For-Final-Review` =
  señal de merge; `S-Needs-Triage` = inbox.
- **Bots por label** (`action-on-PR-labeled.yml`, vía `pull_request_target`
  sin checkout de código no confiable): exigir migration-guide / release-note
  según label. Patrón copiable para "este PR toca X → exige Y".
- **CI-as-code**: crate `tools/ci` (`cargo run -p ci -- lints`); cachés
  precalentadas por schedule (`update-caches.yml`); composite actions propias.
- **NO copiar**: el volumen de proceso (595 PRs, áreas, SMEs) es para una org.
  De bevy nos quedamos con: merge queue + 6–8 labels + un bot de checklist.
  Esfuerzo: **S–M**.

### 3.7 rust-analyzer — QA y triage (PRIORIDAD 2–3)

- **`conclusion` con la trampa documentada**: `skipped == success` para GitHub,
  por eso `if: !cancelled()` + chequeo manual con `jq`, + aviso en mayúsculas
  de añadir cada job nuevo al `needs`. Copiar el comentario, no solo el código.
- **`cargo codegen --check`**: generados frescos como job (nuestro equivalente:
  `git diff --exit-code` tras regenerar).
- **`analysis-stats`**: telemetría de rendimiento EN CI (tiempos del
  analizador). Equivalente nuestro: assert de presupuesto en benches críticos.
- **`cancel-if-matrix-failed`**: cancela la matriz si un eje falla.
- **Feature freeze como herramienta de backlog** (CONTRIBUTING avisa freeze de
  assists). Nuestro `FREEZE_FEATURES` ya existe (Gate 0) — alinear narrativa.
- PRs verdes en **7–8 min**, rojos en **2 min**. Esfuerzo: **S**.

## 4. Lo que NO copiamos

- La escala de proceso de bevy (áreas, SMEs, 60 labels, revisión por capas).
- Matrices exóticas (QEMU/SDE/wasm-tier3 de wasmtime/tokio) hasta tener usuarios
  que las pidan.
- `ty`/`mdtest`/ecosystem-infra de ruff (conformance sobre corpus externo) —
  idea a futuro, no fase 1.
- Daily fuzz / OSS-Fuzz en PR (todos lo sacan del path de merge).

## 5. Backlog de adaptación (fases)

**Fase 0 — quick wins S (una tarde, sin rediseño):**
1. Perfil nextest `ci` (`slow-timeout`, `terminate-after`, resumen de lentos).
2. `cargo insta test --unreferenced reject` en el job de snapshots.
3. `save-if: main` en cachés + `mold`/`line-tables-only` generalizado.
4. Nomenclatura `r<issue>` para nuevos regression tests.
5. Comentario anti-trampa `skipped==success` en nuestro `CI Gate` (estilo ra).

**Fase 1 — tiers (el gran corte, M):**
6. `determine_changes` por áreas (`core/ai/mcp/cli/docs/ci`) + label `no-test`.
7. FAST en PR (fmt, clippy, check, unit-rápidos); FULL (coverage, AI-integration,
   feature-matrix, mutants-weekly, miri, sanitizers, benches-gate) en
   merge queue / main / schedule.
8. Sacar `cargo-mutants` y `Tests (AI integration)` de required; required =
   agregador sobre ~6 jobs rápidos.
9. Tokens `citest:full|miri|ai` para opt-in puntual.

**Fase 2 — merge queue (S–M, desbloquea `strict` sin dolor):**
10. Activar merge queue nativa + `merge_group` en workflows; quitar la
    necesidad de "update branch" manual (el script `merge-when-green.sh`
    sigue valiendo como espera activa).
11. `file-issue-on-error` para los tiers schedule/dispatch.

**Fase 3 — pirámide de tests (M, la auditoría pendiente):**
12. Clasificar los 2744 tests (unit / integration-mock / behavioral-corpus /
    lentos-redundantes) y fijar presupuesto por nivel.
13. Corpus file-driven con stages (idea `testit`), tope de tamaño a snapshots,
    política assertions-first.
14. `cargo codegen --check` / `git diff --exit-code` para generados.

**Fase 4 — cultura PR/issues (continuo, S):**
15. 6–8 labels (`needs-triage`, `ready`, áreas…) + bot de checklist por label
    (estilo bevy, vía `pull_request_target`).
16. Guía CONTRIBUTING: PRs pequeños, diseño-antes, CI-verde-antes-de-review,
    cierre de PRs estancados.
17. Triage programado ligero (labeler por paths).

## 6. Reproducir este estudio

```bash
# Inventario de workflows
gh api repos/<owner>/<repo>/contents/.github/workflows --jq '.[].name'
# CI crudo
gh api repos/<owner>/<repo>/contents/.github/workflows/<file> --jq '.content' | base64 -d
# Wall-clocks (reintentar: la red a api.github.com es intermitente)
gh run list -R <owner>/<repo> --workflow=<file> --limit 8 \
  --json databaseId,event,headBranch,createdAt,updatedAt,conclusion
# Labels / merged recents / dir listing
gh label list -R <owner>/<repo> --limit 60 --json name
gh pr list -R <owner>/<repo> --state merged --limit 5 --json number,title,mergedAt
gh api repos/<owner>/<repo>/contents/<dir> --jq '.[].name'
# Archivos sueltos sin API (canal fiable cuando gh falla):
curl -s "https://raw.githubusercontent.com/<owner>/<repo>/<rama>/<path>"
```
