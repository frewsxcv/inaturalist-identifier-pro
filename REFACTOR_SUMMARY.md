# Refactoring Summary: Crate Separation

This document summarizes the refactoring work that separated the monolithic `inaturalist-pro` crate into multiple focused crates.

## What Changed

### Before
The project had a partial separation with these crates:
- `geo-ext` - Geographic utilities
- `inaturalist-fetch` - Data fetching
- `inaturalist-pro-core` - Core data structures (minimal)
- `inaturalist-pro-ui` - UI components
- `inaturalist-pro` - Everything else (actors, config, geohash logic, operations, main app)

### After
The project now has a clean separation with these crates:
- `geo-ext` - Geographic utilities (unchanged)
- `inaturalist-fetch` - Data fetching (unchanged)
- `inaturalist-pro-config` - ✨ NEW: Configuration management
- `inaturalist-pro-core` - Core data structures (unchanged)
- `inaturalist-pro-geo` - ✨ NEW: Geographic/geohash/places logic
- `inaturalist-pro-actors` - ✨ NEW: Actor system
- `inaturalist-pro-ui` - UI components (unchanged)
- `inaturalist-pro` - Main application (streamlined)

## Files Moved

### Created `inaturalist-pro-config` crate
**Purpose**: Configuration file management and token handling

**New files**:
- `src/lib.rs` - Config struct with load/save/token validation methods

**Extracted from**: `inaturalist-pro/src/main.rs`
- Moved `MyConfig` struct (renamed to `Config`)
- Added helper methods for token management

### Created `inaturalist-pro-geo` crate
**Purpose**: Geographic and geohash functionality

**Files moved from** `inaturalist-pro/src/`:
- `geohash_ext.rs` → `inaturalist-pro-geo/src/geohash_ext.rs`
- `geohash_observations.rs` → `inaturalist-pro-geo/src/geohash_observations.rs`
- `places.rs` → `inaturalist-pro-geo/src/places.rs`

**New file**:
- `src/lib.rs` - Public API exports and type aliases

### Created `inaturalist-pro-actors` crate
**Purpose**: Actor system for concurrent operations

**Files moved from** `inaturalist-pro/src/actors/`:
- `identify_actor.rs` → `inaturalist-pro-actors/src/identify_actor.rs`
- `oauth_actor.rs` → `inaturalist-pro-actors/src/oauth_actor.rs` (updated)
- `observation_loader_actor.rs` → `inaturalist-pro-actors/src/observation_loader_actor.rs` (refactored)
- `observation_processor_actor.rs` → `inaturalist-pro-actors/src/observation_processor_actor.rs` (refactored)
- `taxa_loader_actor.rs` → `inaturalist-pro-actors/src/taxa_loader_actor.rs`
- `taxon_tree_builder_actor.rs` → `inaturalist-pro-actors/src/taxon_tree_builder_actor.rs`
- `mod.rs` → deleted (logic moved to `lib.rs`)

**New file**:
- `src/lib.rs` - Public API exports

### Updated `inaturalist-pro` (main crate)
**Files kept** (with updates):
- `src/main.rs` - Simplified, now uses new crates
- `src/app.rs` - Updated imports to use new crates
- `src/operations.rs` - Updated to use `inaturalist_pro_core::AppMessage`
- `src/utils.rs` - Kept in main crate (could be moved to a utility crate later)

**Files removed**:
- `src/actors/` directory (entire directory moved to `inaturalist-pro-actors`)
- `src/geohash_ext.rs` (moved to `inaturalist-pro-geo`)
- `src/geohash_observations.rs` (moved to `inaturalist-pro-geo`)
- `src/places.rs` (moved to `inaturalist-pro-geo`)

## Code Changes

### Import Updates

**Before** (in `main.rs`):
```rust
use crate::actors::{IdentifyActor, OauthActor, ...};
use crate::geohash_ext::GeohashGrid;
use crate::places;
```

**After**:
```rust
use inaturalist_pro_actors::{IdentifyActor, OauthActor, ...};
use inaturalist_pro_geo::{GeohashGrid, places};
use inaturalist_pro_config::Config;
```

### Configuration Loading

**Before** (in `main.rs`):
```rust
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
struct MyConfig {
    token: Option<TokenDetails>,
}

let cfg: MyConfig = confy::load("inaturalist-identifier-pro", None)?;
let api_token = if let Some(token) = cfg.token {
    if token.expires_at >= std::time::SystemTime::now() {
        Some(token.api_token)
    } else {
        None
    }
} else {
    None
};
```

**After**:
```rust
use inaturalist_pro_config::Config;

let cfg = Config::load()?;
let api_token = cfg.get_api_token();
```

### Actor Updates

**`ObservationLoaderActor`** now includes the request parameters and soft limit:
```rust
pub struct ObservationLoaderActor {
    pub tx_app_message: UnboundedSender<AppMessage>,
    pub grid: GeohashGrid,
    pub api_token: String,
    pub request: inaturalist::apis::observations_api::ObservationsGetParams, // Added
    pub soft_limit: std::sync::Arc<std::sync::atomic::AtomicI32>,           // Added
}
```

**`ObservationProcessorActor`** simplified to always fetch CV scores:
- Removed the `operation` field
- Directly fetches computer vision scores for all observations
- Removed dependency on the `Operation` trait

**`OauthActor`** updated to use a helper function for config saving:
- Uses a local helper to avoid circular dependencies with config crate
- Could be improved to use `inaturalist-pro-config` crate directly

## Dependency Changes

### Main `inaturalist-pro` crate dependencies

**Removed** (now in other crates):
- `arrayvec`
- `colorous` (used by UI)
- `egui_extras` (used by UI)
- `futures`
- `genawaiter`
- `geo`
- `geohash`
- `geojson`
- `image` (used by UI)
- `reqwest` (used by fetch)
- `ordered-float`
- `serde_json`

**Added** (new crate dependencies):
- `inaturalist-pro-actors`
- `inaturalist-pro-config`
- `inaturalist-pro-geo`

**Kept**:
- `actix` (still orchestrates actors)
- `eframe` (app framework)
- `egui` (for types in app.rs)
- `inaturalist`
- `inaturalist-fetch` (used directly in operations.rs)
- `inaturalist-oauth`
- `oauth2`
- `opener`
- `serde`
- `tokio`
- `tracing`
- `tracing-subscriber`

## Breaking Changes

### For External Consumers (if any)

If anyone was depending on `inaturalist-pro` as a library (unlikely), they would need to:

1. Update imports from `inaturalist_pro::` to the appropriate new crate
2. Add dependencies on the new crates they use
3. Update `Cargo.toml` to include the new crates

Example:
```toml
[dependencies]
inaturalist-pro-core = { path = "../inaturalist-pro/inaturalist-pro-core" }
inaturalist-pro-geo = { path = "../inaturalist-pro/inaturalist-pro-geo" }
inaturalist-pro-actors = { path = "../inaturalist-pro/inaturalist-pro-actors" }
```

### For Internal Development

No breaking changes for running the application:
- Binary name remains `inaturalist-pro`
- Configuration file location unchanged (`~/Library/Application Support/rs.inaturalist-identifier-pro/`)
- Runtime behavior identical

## Benefits Achieved

1. **Clearer Code Organization**
   - Each crate has a single, well-defined purpose
   - Easy to find where specific functionality lives

2. **Improved Build Times**
   - Smaller compilation units
   - Only affected crates rebuild on changes
   - Parallel compilation of independent crates

3. **Better Testability**
   - Can test crates in isolation
   - Easier to mock dependencies
   - Clearer test boundaries

4. **Reduced Coupling**
   - Explicit dependencies via `Cargo.toml`
   - Harder to create circular dependencies
   - Clear dependency hierarchy

5. **Enhanced Reusability**
   - Core crates can be used in other projects
   - UI-less applications can use just actors + core
   - Testing tools can use fetch + core without UI

6. **Simplified Onboarding**
   - New developers can understand one crate at a time
   - Documentation can be more focused
   - Clear boundaries reduce cognitive load

## Migration Checklist

If you're working with an old branch, here's what to do:

- [ ] Pull latest changes from main branch
- [ ] Clean build artifacts: `cargo clean`
- [ ] Update imports in your code:
  - [ ] Replace `crate::actors::*` with `inaturalist_pro_actors::*`
  - [ ] Replace `crate::geohash_ext::*` with `inaturalist_pro_geo::*`
  - [ ] Replace `crate::places::*` with `inaturalist_pro_geo::places::*`
  - [ ] Replace `MyConfig` with `inaturalist_pro_config::Config`
- [ ] Update `use` statements to use the new crate paths
- [ ] Run `cargo check` to find any remaining issues
- [ ] Run `cargo test` to ensure tests pass
- [ ] Run `cargo build` to verify everything compiles

## Future Work

Potential next steps for further modularization:

1. **Extract operations**
   - Create `inaturalist-pro-operations` crate
   - Move `operations.rs` and the `Operation` trait
   - Make operations pluggable

2. **Split UI crate**
   - Separate views into individual crates
   - Extract common widgets to `inaturalist-pro-widgets`
   - Create theme/styling crate

3. **Add workspace-level tests**
   - Integration tests across crates
   - End-to-end testing
   - Performance benchmarks

4. **Improve config crate**
   - Add migration support for config format changes
   - Add validation and schema checking
   - Support multiple config profiles

5. **Documentation improvements**
   - Add `cargo doc` comments to all public APIs
   - Create examples for each crate
   - Add architecture diagrams

## Questions or Issues?

If you encounter problems after this refactoring:

1. Check `CRATE_STRUCTURE.md` for the current architecture
2. Review this document for migration guidance
3. Look at the dependency graph to understand relationships
4. Check git history for specific file moves: `git log --follow <filename>`

## Credits

This refactoring was completed to improve code maintainability and prepare the codebase for future growth. The separation follows Rust best practices for workspace organization.

---

**Refactoring Date**: 2024
**Version After Refactor**: 0.1.0

## Post-Refactoring Fixes

### Image Loaders Warning

After the refactoring, a warning appeared:
```
WARN egui_extras::loaders: `install_image_loaders` was called, but no loaders are enabled
```

**Fix Applied**: Updated `inaturalist-pro-ui/Cargo.toml` to enable image loaders:
```toml
egui_extras = { version = "0.32", features = ["all_loaders"] }
image = { version = "0.25", features = ["jpeg", "png"] }
```

This ensures that `egui_extras::install_image_loaders()` has the necessary image format support enabled.