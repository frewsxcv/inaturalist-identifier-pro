# iNaturalist Pro - Application Structure

## Visual Navigation Structure

```
┌─────────────────────────────────────────────────────────────────────────┐
│                          iNaturalist Pro                                │
├─────────────┬───────────────────────────────────────────────────────────┤
│             │                                                             │
│  NAVIGATION │                     MAIN CONTENT AREA                      │
│   SIDEBAR   │                                                             │
│             │                                                             │
│ ┌─────────┐ │  ┌───────────────────────────────────────────────────┐   │
│ │🔍 Identify│ │  │                                                   │   │
│ └─────────┘ │  │  Currently Selected View Displays Here            │   │
│             │  │                                                   │   │
│ ┌─────────┐ │  │  - Identify: Observation gallery + ID panels      │   │
│ │📷 Observ.│ │  │  - Observations: Search form + results grid       │   │
│ └─────────┘ │  │  - Users: Search + profile display                │   │
│             │  │  - Taxa: Search sidebar + taxon details           │   │
│ ┌─────────┐ │  │                                                   │   │
│ │👤 Users  │ │  └───────────────────────────────────────────────────┘   │
│ └─────────┘ │                                                             │
│             │                                                             │
│ ┌─────────┐ │                                                             │
│ │🌿 Taxa   │ │                                                             │
│ └─────────┘ │                                                             │
│             │                                                             │
└─────────────┴───────────────────────────────────────────────────────────┘
```

## Code Structure

```
inaturalist-pro/
│
├── inaturalist_pro/            # Main GUI application
│   └── src/
│       ├── main.rs             # Application entry point
│       ├── app.rs              # Main app logic & navigation
│       │
│       ├── views/              # View modules (one per section)
│       │   ├── mod.rs
│       │   ├── identify_view.rs       ✓ Fully functional
│       │   ├── observations_view.rs   ⚠ UI only (needs API)
│       │   ├── users_view.rs          ⚠ UI only (needs API)
│       │   └── taxa_view.rs           ⚠ UI only (needs API)
│       │
│       ├── panels/             # Reusable UI components
│       │   ├── details_panel.rs
│       │   ├── identification_panel.rs
│       │   ├── observation_gallery_panel.rs
│       │   └── top_panel.rs
│       │
│       ├── actors/             # Background data fetchers
│       │   ├── identify_actor.rs
│       │   ├── observation_loader_actor.rs
│       │   ├── observation_processor_actor.rs
│       │   ├── taxa_loader_actor.rs
│       │   └── taxon_tree_builder_actor.rs
│       │
│       └── [other modules...]
│
├── inaturalist-fetch/          # iNaturalist API client library
└── geo-ext/                    # Geographic utilities
```

## Data Flow Diagram

```
┌──────────┐
│   User   │
│  Action  │
└────┬─────┘
     │
     v
┌─────────────┐      ┌──────────────┐      ┌─────────────┐
│    View     │─────>│    Actor     │─────>│ iNaturalist │
│  (UI Layer) │      │  (Async I/O) │      │     API     │
└─────────────┘      └──────────────┘      └─────────────┘
     ^                      │                       
     │                      │                       
     │               ┌──────┴───────┐               
     │               │   Message    │               
     └───────────────┤   Channel    │               
                     └──────────────┘               
```

## View Details

### 🔍 Identify View

**Purpose**: Browse and identify unidentified observations

**Components**:
- Observation Gallery Panel (thumbnails)
- Identification Panel (taxa tree, CV scores)
- Details Panel (observation metadata)

**State**:
- List of loaded observations
- Current observation selection
- Computer vision scores
- Taxonomic trees

**Status**: ✅ Fully implemented and functional

---

### 📷 Observations View

**Purpose**: Query and browse iNaturalist observations

**Layout**:
```
┌─────────────────────────────────────────┐
│  Search Form                            │
│  ├─ Search query                        │
│  ├─ Taxon filter                        │
│  ├─ User filter                         │
│  ├─ Place filter                        │
│  ├─ Date range                          │
│  ├─ Quality grade                       │
│  └─ Identified filter                   │
│                                         │
│  [Search] [Clear]                       │
├─────────────────────────────────────────┤
│  Results Grid                           │
│  ID | Taxon | User | Date | Place      │
│  ────────────────────────────────────   │
│  ... observation rows ...               │
└─────────────────────────────────────────┘
```

**Status**: ⚠️ UI implemented, needs API integration

---

### 👤 Users View

**Purpose**: Look up and view user profiles

**Layout**:
```
┌─────────────────────────────────────────┐
│  Username: [________] [Search] [Clear]  │
├─────────────────────────────────────────┤
│  User Profile                           │
│  ┌─────┐                                │
│  │ 🖼️  │ username                       │
│  │     │ Real Name                      │
│  └─────┘ User ID: 12345                 │
│                                         │
│  Statistics                             │
│  📷 Observations:     1,234             │
│  🔍 Identifications:  5,678             │
│  🌿 Species:          890               │
│  📝 Journal Posts:    12                │
│  💫 Total Activity:   6,924             │
│                                         │
│  Details                                │
│  Member since: 2020-01-15               │
│  Site ID: 1                             │
└─────────────────────────────────────────┘
```

**Status**: ⚠️ UI implemented, needs API integration

---

### 🌿 Taxa View

**Purpose**: Explore taxonomy and search for organisms

**Layout**:
```
┌──────────────┬─────────────────────────────┐
│ Search Panel │  Taxon Details              │
│              │                             │
│ Taxon name:  │  Animalia (Kingdom)         │
│ [________]   │  Animals                    │
│              │  Taxon ID: 1                │
│ Rank: [Any▼] │                             │
│              │  📷 Observations: 50,000,000│
│ [Search]     │  🏷️ Iconic Taxon: Animalia  │
│ [Clear]      │                             │
│              │  Taxonomy                   │
│ ──────────── │  Kingdom: Animalia ✓        │
│              │                             │
│ 3 results    │  About                      │
│              │  Animals are multicellular  │
│ • Animalia   │  eukaryotic organisms...    │
│   Animals    │                             │
│              │                             │
│ • Plantae    │                             │
│   Plants     │                             │
│              │                             │
│ • Fungi      │                             │
│   Fungi      │                             │
└──────────────┴─────────────────────────────┘
```

**Status**: ⚠️ UI implemented, needs API integration

---

## State Management

```
┌─────────────────────────────────────────┐
│              AppState                   │
├─────────────────────────────────────────┤
│ - current_view: AppView                 │
│ - loaded_geohashes: usize               │
│ - results: Vec<QueryResult>             │
│ - taxa_store: TaxaStore                 │
│ - current_observation_id: Option<i32>   │
└─────────────────────────────────────────┘
           │
           ├─────────────┬─────────────┬─────────────┐
           │             │             │             │
           v             v             v             v
    ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐
    │ Identify │  │Observ.   │  │  Users   │  │   Taxa   │
    │   View   │  │  View    │  │   View   │  │   View   │
    └──────────┘  └──────────┘  └──────────┘  └──────────┘
    (own state)   (own state)   (own state)   (own state)
```

## Message Flow

```
User Interaction
      │
      v
┌─────────────┐
│    View     │
│  Component  │
└──────┬──────┘
       │
       │ Triggers action
       v
┌─────────────┐
│   Actor     │◄─────── Arbiter (background thread)
│  (Actix)    │
└──────┬──────┘
       │
       │ Fetches data
       v
┌─────────────┐
│ iNaturalist │
│     API     │
└──────┬──────┘
       │
       │ Returns data
       v
┌─────────────┐
│  Message    │
│  Channel    │
└──────┬──────┘
       │
       v
┌─────────────┐
│ App::       │
│ process_    │
│ messages()  │
└──────┬──────┘
       │
       │ Updates state
       v
┌─────────────┐
│    View     │
│   Re-render │
└─────────────┘
```

## Key Files

| File | Purpose | Status |
|------|---------|--------|
| `main.rs` | Entry point, actor setup, auth | ✅ Complete |
| `app.rs` | Main app logic, navigation | ✅ Complete |
| `views/identify_view.rs` | Identification interface | ✅ Complete |
| `views/observations_view.rs` | Observation query UI | ⚠️ Needs API |
| `views/users_view.rs` | User lookup UI | ⚠️ Needs API |
| `views/taxa_view.rs` | Taxonomy explorer UI | ⚠️ Needs API |
| `actors/*.rs` | Background data fetchers | ✅ Complete |
| `panels/*.rs` | Reusable UI components | ✅ Complete |

## Navigation Implementation

The navigation is implemented using egui's `selectable_value`:

```rust
// In app.rs render_ui()
egui::SidePanel::left("navigation_panel")
    .show(ctx, |ui| {
        ui.heading("iNaturalist Pro");
        ui.separator();
        
        ui.selectable_value(&mut self.state.current_view, AppView::Identify, "🔍 Identify");
        ui.selectable_value(&mut self.state.current_view, AppView::Observations, "📷 Observations");
        ui.selectable_value(&mut self.state.current_view, AppView::Users, "👤 Users");
        ui.selectable_value(&mut self.state.current_view, AppView::Taxa, "🌿 Taxa");
    });

match self.state.current_view {
    AppView::Identify => self.render_identify_view(ctx),
    AppView::Observations => self.render_observations_view(ctx),
    AppView::Users => self.render_users_view(ctx),
    AppView::Taxa => self.render_taxa_view(ctx),
}
```

## Future Architecture Considerations

1. **Shared Components**: Extract common search/filter UI into reusable components
2. **State Persistence**: Save user preferences and view state between sessions
3. **Caching Layer**: Cache API responses to reduce network calls
4. **Offline Support**: Store data locally for offline browsing
5. **Plugin System**: Allow community-contributed views/features