# iNaturalist Pro - Project Structure

This document provides an overview of the iNaturalist Pro project structure after the modular refactoring.

## Quick Navigation

- [Crate Structure](#crate-structure) - Overview of all crates
- [CRATE_STRUCTURE.md](CRATE_STRUCTURE.md) - Detailed crate documentation
- [REFACTOR_SUMMARY.md](REFACTOR_SUMMARY.md) - What changed in the refactor

## Workspace Overview

The project is organized as a Cargo workspace with 8 crates:

```
inaturalist-pro/
├── Cargo.toml                          # Workspace definition
├── Cargo.lock                          # Dependency lock file
│
├── geo-ext/                            # Geographic type extensions
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs
│
├── inaturalist-fetch/                  # API data fetching
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs
│
├── inaturalist-pro-config/             # Configuration management
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs
│
├── inaturalist-pro-core/               # Core data structures
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs
│
├── inaturalist-pro-geo/                # Geographic/geohash logic
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── geohash_ext.rs
│       ├── geohash_observations.rs
│       └── places.rs
│
├── inaturalist-pro-actors/             # Actor system
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── identify_actor.rs
│       ├── oauth_actor.rs
│       ├── observation_loader_actor.rs
│       ├── observation_processor_actor.rs
│       ├── taxa_loader_actor.rs
│       └── taxon_tree_builder_actor.rs
│
├── inaturalist-pro-ui/                 # UI components
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── panels/
│       │   ├── mod.rs
│       │   └── top_panel.rs
│       ├── views/
│       │   ├── mod.rs
│       │   ├── identify_view.rs
│       │   ├── observations_view.rs
│       │   ├── taxa_view.rs
│       │   └── users_view.rs
│       ├── widgets/
│       │   ├── mod.rs
│       │   └── taxon_tree.rs
│       └── utils.rs
│
└── inaturalist-pro/                    # Main application
    ├── Cargo.toml
    └── src/
        ├── main.rs                     # Entry point
        ├── app.rs                      # App struct & eframe impl
        ├── operations.rs               # Query operations
        └── utils.rs                    # Utility functions
```

## Crate Responsibilities

### Foundation Layer
- **geo-ext**: Low-level geographic utilities
- **inaturalist-fetch**: API communication and data fetching

### Core Layer
- **inaturalist-pro-config**: Configuration file management
- **inaturalist-pro-core**: Shared data structures and types
- **inaturalist-pro-geo**: Geographic and geohash functionality

### Service Layer
- **inaturalist-pro-actors**: Concurrent task management with Actix

### Presentation Layer
- **inaturalist-pro-ui**: All UI components and views

### Application Layer
- **inaturalist-pro**: Main binary that ties everything together

## Data Flow

```
User Input (egui)
    ↓
inaturalist-pro-ui
    ↓
AppMessage → inaturalist-pro (app.rs)
    ↓
Actor Messages → inaturalist-pro-actors
    ↓
API Requests → inaturalist-fetch
    ↓
AppMessage → inaturalist-pro (app.rs)
    ↓
State Update → inaturalist-pro-core (AppState)
    ↓
UI Update → inaturalist-pro-ui
```

## Key Types and Their Locations

| Type | Crate | Purpose |
|------|-------|---------|
| `AppState` | core | Central application state |
| `AppMessage` | core | Inter-component messages |
| `TaxonTree` | core | Taxonomic hierarchy |
| `Config` | config | Configuration management |
| `Geohash` | geo | Geographic hash representation |
| `GeohashGrid` | geo | Grid of geohashes |
| `IdentifyActor` | actors | Submits identifications |
| `ObservationLoaderActor` | actors | Loads observations |
| `Ui<T>` | ui | Main UI coordinator |

## Actor System Architecture

The application uses Actix actors for concurrent operations:

```
┌─────────────────────────────────────────────────────┐
│                   Main Thread                       │
│  ┌───────────────────────────────────────────────┐ │
│  │         App (eframe::App)                     │ │
│  │  - Processes AppMessages                      │ │
│  │  - Updates AppState                           │ │
│  │  - Renders UI                                 │ │
│  └───────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────┘
                      ↓ ↑
                AppMessage channel
                      ↓ ↑
┌─────────────────────────────────────────────────────┐
│                Actor System (Actix)                 │
│                                                     │
│  ┌─────────────────────────────────────────────┐  │
│  │ ObservationLoaderActor                      │  │
│  │ - Loads observations by geohash grid        │  │
│  └─────────────────────────────────────────────┘  │
│                      ↓                              │
│  ┌─────────────────────────────────────────────┐  │
│  │ ObservationProcessorActor                   │  │
│  │ - Processes observations                    │  │
│  │ - Fetches CV scores                         │  │
│  └─────────────────────────────────────────────┘  │
│                      ↓                              │
│  ┌─────────────────────────────────────────────┐  │
│  │ TaxonTreeBuilderActor                       │  │
│  │ - Builds taxonomic trees                    │  │
│  └─────────────────────────────────────────────┘  │
│                      ↓                              │
│  ┌─────────────────────────────────────────────┐  │
│  │ TaxaLoaderActor                             │  │
│  │ - Batch loads taxon information             │  │
│  └─────────────────────────────────────────────┘  │
│                                                     │
│  ┌─────────────────────────────────────────────┐  │
│  │ IdentifyActor                               │  │
│  │ - Submits identifications to iNaturalist    │  │
│  └─────────────────────────────────────────────┘  │
│                                                     │
│  ┌─────────────────────────────────────────────┐  │
│  │ OauthActor                                  │  │
│  │ - Handles OAuth token exchange              │  │
│  └─────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────┘
```

## Build and Development

### Building the Project

```bash
# Build all crates
cargo build

# Build with optimizations
cargo build --release

# Build specific crate
cargo build -p inaturalist-pro-core

# Check all crates without building
cargo check --workspace
```

### Running the Application

```bash
# Run in development mode
cargo run

# Run with release optimizations
cargo run --release

# Run with logging
RUST_LOG=debug cargo run
```

### Testing

```bash
# Test all crates
cargo test --workspace

# Test specific crate
cargo test -p inaturalist-pro-core

# Run tests with output
cargo test -- --nocapture
```

### Documentation

```bash
# Generate documentation for all crates
cargo doc --workspace --no-deps --open

# Generate documentation for a specific crate
cargo doc -p inaturalist-pro-core --open
```

## Configuration

Configuration is stored at:
- **macOS**: `~/Library/Application Support/rs.inaturalist-identifier-pro/default-config.toml`
- **Linux**: `~/.config/rs.inaturalist-identifier-pro/default-config.toml`
- **Windows**: `%APPDATA%\rs.inaturalist-identifier-pro\default-config.toml`

### Configuration Format

```toml
[token]
api_token = "your_api_token_here"
token_type = "Bearer"
expires_at = { secs_since_epoch = 1234567890, nanos_since_epoch = 0 }
```

## Common Development Tasks

### Adding a New Actor

1. Create actor file in `inaturalist-pro-actors/src/my_actor.rs`
2. Define actor struct and messages
3. Implement `Actor` trait and message handlers
4. Export from `inaturalist-pro-actors/src/lib.rs`
5. Register in `inaturalist-pro/src/main.rs`

### Adding a New UI View

1. Create view file in `inaturalist-pro-ui/src/views/my_view.rs`
2. Implement view struct with `show()` method
3. Export from `inaturalist-pro-ui/src/views/mod.rs`
4. Add to `AppViews` struct in `inaturalist-pro-ui/src/lib.rs`
5. Add view enum variant to `AppView` in `inaturalist-pro-core/src/lib.rs`

### Adding a New Message Type

1. Add variant to `AppMessage` enum in `inaturalist-pro-core/src/lib.rs`
2. Handle in `App::process_messages()` in `inaturalist-pro/src/app.rs`
3. Send from actors or UI as needed

## Troubleshooting

### Token Mismatch Error

If you see a TOML parsing error about token format:

```bash
# Backup and reset config
mv ~/Library/Application\ Support/rs.inaturalist-identifier-pro \
   ~/Library/Application\ Support/rs.inaturalist-identifier-pro.backup
```

Then restart the app and authenticate again.

### Build Errors After Pulling Changes

```bash
# Clean and rebuild
cargo clean
cargo build
```

### Actor System Issues

Check logs with:
```bash
RUST_LOG=actix=debug,inaturalist_pro=debug cargo run
```

## Additional Documentation

- [CRATE_STRUCTURE.md](CRATE_STRUCTURE.md) - Detailed crate documentation
- [REFACTOR_SUMMARY.md](REFACTOR_SUMMARY.md) - Refactoring details
- [AUTHENTICATION_CHANGELOG.md](AUTHENTICATION_CHANGELOG.md) - Auth system changes
- [AUTHENTICATION_QUICK_START.md](AUTHENTICATION_QUICK_START.md) - Auth setup guide
- [TROUBLESHOOTING.md](TROUBLESHOOTING.md) - Common issues and solutions
- [USER_GUIDE.md](USER_GUIDE.md) - User documentation

## Contributing

When contributing code:

1. Keep crate responsibilities clear - don't blur boundaries
2. Update documentation when adding features
3. Run tests before committing: `cargo test --workspace`
4. Check for warnings: `cargo clippy --workspace`
5. Format code: `cargo fmt --all`

## License

See LICENSE file for details.

---

**Last Updated**: 2024
**Version**: 0.1.0