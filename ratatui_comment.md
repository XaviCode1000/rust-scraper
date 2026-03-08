## 🎨 Actualización: TUI con Ratatui

Se ha decidido utilizar **Ratatui** para la implementación del TUI interactivo (Fase 2).

### Razones:
- ✅ Activo y mantenido (basado en tui-rs)
- ✅ Cross-platform (Linux, macOS, Windows)
- ✅ Widgets ricos (List, Table, Tabs, Scrollbar)
- ✅ Layout system tipo Flexbox
- ✅ Event handling completo (teclado, mouse, resize)
- ✅ Templates disponibles (`cargo generate ratatui/templates`)

### Dependencies actualizadas:

```toml
[dependencies]
# TUI
ratatui = "0.29"
crossterm = "0.28"
```

### Preview del UI:

```
┌────────────────────────────────────────────────┐
│           🕷️ Rust Scraper - Selecciona URLs   │
├────────────────────────────────────────────────┤
│ ┌────────────────────────────────────────────┐ │
│ │ URLs                                       │ │
│ │ 👉 ✓ https://docs.ejemplo.com/             │ │
│ │    ✓ https://docs.ejemplo.com/guide/       │ │
│ │      https://docs.ejemplo.com/api/         │ │
│ │    ✓ https://docs.ejemplo.com/blog/        │ │
│ └────────────────────────────────────────────┘ │
├────────────────────────────────────────────────┤
│ ↑↓: Navegar | Space: Seleccionar | Enter: OK  │
└────────────────────────────────────────────────┘
```

**Controls:**
- `↑↓` - Navegar
- `Space` - Seleccionar/deseleccionar
- `a` - Seleccionar todos
- `n` - Deseleccionar todos  
- `Enter` - Descargar seleccionadas
- `q` - Salir
- `Ctrl+C` - Cancelar

---

Esta actualización se aplica a la **Fase 2** de la issue.
