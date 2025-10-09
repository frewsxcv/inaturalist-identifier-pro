# iNaturalist Pro - Crate Structure

This document describes the modular architecture of the iNaturalist Pro application after the refactoring to separate concerns into distinct crates.

## Overview

The project is organized as a Cargo workspace with multiple crates, each with a specific responsibility. This separation improves code organization, compilation times, testability, and makes it easier to reason about dependencies.

## Crate Hierarchy

```
inaturalist-pro/
├── geo-ext/                    # Geographic utilities
├── inaturalist-fetch/          # API data fetching
├── inaturalist-pro-config/     # Configuration management
├── inaturalist-pro-core/       # Core data structures
├── inaturalist-pro-geo/        # Geographic/geohash logic
├── inaturalist-pro-actors/     # Actor system
├── inaturalist-pro-ui/         # UI components
└── inaturalist-pro/            # Main application binary
```

## Crate Details

### `geo-ext`
**Purpose**: Low-level geographic type extensions

**Responsibilities**:
- Extensions for the `geo` crate types
- Geographic utility functions

**Dependencies**: `geo`

**Depended on by**: `inaturalist-fetch`, `inaturalist-pro-geo`

---

### `inaturalist-fetch`
**Purpose**: Data fetching from iNaturalist API

**Responsibilities**:
- Fetching observations, taxa, and other data from iNaturalist API
- Rate limiting with `governor`
- Recursive subdivision of geographic areas for efficient data retrieval
- Computer vision score fetching

**Key dependencies**: `inaturalist`, `reqwest`, `tokio`, `geo-ext`

**Depended on by**: `inaturalist-pro-geo`, `inaturalist-pro-actors`, `inaturalist-pro`

---

### `inaturalist-pro-config`
**Purpose**: Application configuration management

**Responsibilities**:
- Configuration file loading and saving
- Token storage and validation
- Token expiration checking
- Configuration migrations (future)

**Key types**:
- `Config`: Main configuration struct with token management methods

**Key dependencies**: `confy`, `inaturalist-oauth`, `serde`

**Depended on by**: `inaturalist-pro`

---

### `inaturalist-pro-core`
**Purpose**: Core data structures and types shared across all crates

**Responsibilities**:
- Core domain models (`Taxon`, `TaxaStore`, `TaxonTree`)
- Application state (`AppState`)
- Message passing types (`AppMessage` enum)
- Query results and view enums
- Type aliases used throughout the application

**Key types**:
- `AppMessage`: Message enum for actor communication
- `AppState`: Central application state
- `TaxonTree`: Tree structure for taxonomic hierarchies
- `QueryResult`: Observation results with scores and trees
- `AppView`: Enum for different UI views

**Key dependencies**: `inaturalist`, `inaturalist-fetch`, `oauth2`, `serde`

**Depended on by**: All other `inaturalist-pro-*` crates

---

### `inaturalist-pro-geo`
**Purpose**: Geographic and geohash-specific logic

**Responsibilities**:
- Geohash grid generation and manipulation
- Observation fetching by geographic area
- Place definitions (NYC, Brooklyn, Prospect Park, etc.)
- Geographic rectangle utilities

**Key types**:
- `Geohash`: Geohash string with bounding rectangle
- `GeohashGrid`: Collection of geohashes covering an area
- `GeohashObservations`: Fetching observations within a geohash
- `Rect`: Type alias for ordered-float rectangles

**Key modules**:
- `geohash_ext`: Geohash and grid implementations
- `geohash_observations`: Observation fetching by geohash
- `places`: Predefined geographic areas

**Key dependencies**: `geo`, `geohash`, `inaturalist`, `inaturalist-fetch`

**Depended on by**: `inaturalist-pro-actors`, `inaturalist-pro`

---

### `inaturalist-pro-actors`
**Purpose**: Actix actor system for concurrent operations

**Responsibilities**:
- Actor definitions for various background tasks
- Message handling between actors
- Asynchronous data processing

**Key actors**:
- `IdentifyActor`: Submits identifications to iNaturalist
- `OauthActor`: Handles OAuth token exchange
- `ObservationLoaderActor`: Loads observations from API by geohash grid
- `ObservationProcessorActor`: Processes observations and fetches CV scores
- `TaxaLoaderActor`: Batch loads taxa information
- `TaxonTreeBuilderActor`: Builds taxonomic trees from CV scores

**Key dependencies**: `actix`, `inaturalist`, `inaturalist-fetch`, `inaturalist-oauth`, `inaturalist-pro-core`, `inaturalist-pro-geo`

**Depended on by**: `inaturalist-pro`

---

### `inaturalist-pro-ui`
**Purpose**: User interface components and widgets

**Responsibilities**:
- egui-based UI components
- View implementations (Identify, Observations, Taxa, Users)
- Reusable widgets (taxon tree, observation cards)
- Panel implementations (top panel)

**Key modules**:
- `views/`: Different application views
  - `identify_view`: Main identification interface
  - `observations_view`: Observation browsing
  - `taxa_view`: Taxon exploration
  - `users_view`: User management
- `widgets/`: Reusable UI components
  - `taxon_tree`: Tree visualization for taxonomy
  - Other custom widgets
- `panels/`: UI panels
  - `top_panel`: Top menu bar

**Key types**:
- `Ui<T>`: Main UI coordinator struct (generic over actor type)

**Key dependencies**: `egui`, `eframe`, `actix`, `inaturalist-pro-core`

**Depended on by**: `inaturalist-pro`

---

### `inaturalist-pro` (main binary)
**Purpose**: Application entry point and orchestration

**Responsibilities**:
- Application initialization and startup
- Actor system setup and registration
- eframe application loop
- OAuth flow coordination
- Operation definitions (query strategies)
- App state management and message routing

**Key modules**:
- `main.rs`: Entry point, actor setup, config loading
- `app.rs`: Main App struct implementing `eframe::App`
- `operations.rs`: Operation trait and implementations
- `utils.rs`: Utility functions

**Key dependencies**: `actix`, `eframe`, `egui`, `inaturalist-oauth`, all `inaturalist-pro-*` crates

---

## Dependency Flow

```
┌─────────────────────┐
│   geo-ext (util)    │
└──────────┬──────────┘
           │
           ↓
┌─────────────────────┐
│ inaturalist-fetch   │
└──────────┬──────────┘
           │
           ↓
┌─────────────────────┐     ┌──────────────────────┐
│ inaturalist-pro-    │────→│ inaturalist-pro-geo  │
│      core           │     └──────────┬───────────┘
└──────────┬──────────┘                │
           │                            │
           ↓                            ↓
┌─────────────────────┐     ┌──────────────────────┐
│ inaturalist-pro-ui  │     │ inaturalist-pro-     │
└──────────┬──────────┘     │      actors          │
           │                └──────────┬───────────┘
           │                           │
           │         ┌─────────────────┘
           │         │
           ↓         ↓
      ┌────────────────────┐
      │ inaturalist-pro    │
      │   (main binary)    │
      └────────────────────┘
```

## Benefits of This Structure

1. **Separation of Concerns**: Each crate has a clear, focused responsibility
2. **Reusability**: Core crates (`-core`, `-geo`, `-actors`) can be used independently
3. **Faster Compilation**: Changes to one crate only recompile dependent crates
4. **Testability**: Each crate can be tested in isolation
5. **Clear Dependencies**: Dependency graph makes it obvious what depends on what
6. **Modularity**: Easy to add new crates or remove unused functionality

## Future Improvements

- Extract `operations.rs` into `inaturalist-pro-operations` crate
- Consider splitting `inaturalist-pro-ui` into smaller view-specific crates
- Add integration tests at the workspace level
- Document public APIs with `cargo doc`
- Add examples for using library crates independently

## Building and Testing

```bash
# Build all crates
cargo build

# Test all crates
cargo test

# Build a specific crate
cargo build -p inaturalist-pro-core

# Check all crates
cargo check --workspace
```

## Adding a New Crate

1. Create directory: `mkdir inaturalist-pro/my-new-crate`
2. Add to workspace in root `Cargo.toml`:
   ```toml
   [workspace]
   members = [
       # ... existing members
       "my-new-crate",
   ]
   ```
3. Create `Cargo.toml` in the new crate directory
4. Add dependencies to crates that will use it
5. Import and use in dependent crates

---

**Last Updated**: 2024
**Version**: 0.1.0