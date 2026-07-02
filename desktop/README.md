# Loon Admin Desktop

Desktop admin GUI for Loon, built on Nest [`nest-gui`](../../../core/crates/nest-gui) and [`nest-theme`](../../../core/crates/nest-theme).

## Layout

```text
desktop/
├── Cargo.toml                 # workspace (admin crates)
├── build                      # build / run / test helpers
├── config.example.toml        # `[gui]` section for the admin app
├── docs/
│   └── v1.md                  # implementation plan
├── themes/
│   └── loon-dark.toml         # Loon-branded ThemeDefinition
└── crates/
    ├── loon-admin/            # nest-gui binary — admin shell + screens
    └── loon-egui-theme/       # nest-theme → egui Visuals adapter
```

## Nest stack

| Layer | Crate | Role |
|-------|-------|------|
| Host | `nest-gui` | eframe loop, window, logging, config |
| Theme lifecycle | `nest-theme` | `ThemeModule`, `ThemeService` |
| Token schema | `nest-design` | `ThemeDefinition`, color/spacing tokens |
| Loon adapter | `loon-egui-theme` | `ThemeAdapter<egui::Visuals>` for Loon branding |
| Product | `loon-admin` | Admin views, Loon server API client |

## Quick start

```bash
cd apps/loon
cp desktop/config.example.toml desktop/config.toml   # optional
./build desktop run
```

Requires a display (eframe native window). The scaffold opens a placeholder admin shell; screens and server wiring come in v1.

## Related

- [docs/v1.md](docs/v1.md) — phased plan
- [Nest nest-gui docs](../../../docs/nest-gui/README.md)
- [Nest nest-theme docs](../../../docs/nest-theme/README.md)
