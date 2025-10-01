# Navigation Guide for iNaturalist Pro

This document describes the navigation structure and architecture of the iNaturalist Pro application.

## Overview

iNaturalist Pro has been restructured from a single-purpose identification tool into a general-purpose iNaturalist application with multiple sections. Users can navigate between different aspects of iNaturalist through a sidebar navigation menu.

## Navigation Structure

The application features a left sidebar with four main sections:

### 🔍 Identify
The original identification interface, now one of several features. This section allows users to:
- Browse unidentified observations
- View computer vision scores for observations
- Access taxonomic information
- Help the community with identifications

**Status**: Fully implemented and functional

### 📷 Observations
A query interface for searching and browsing iNaturalist observations.

**Current Features**:
- Search form with multiple filters:
  - General search query
  - Taxon filter
  - User filter
  - Place filter
  - Date range (from/to)
  - Quality grade (Any, Research, Needs ID, Casual)
  - Identified status (Any, Yes, No)
- Results grid display

**Status**: UI implemented, backend integration needed

**Next Steps**:
1. Connect to the iNaturalist API observations endpoint
2. Implement result pagination
3. Add observation detail view
4. Include observation photos in results

### 👤 Users
A user profile lookup tool for viewing iNaturalist user information.

**Current Features**:
- Username search
- User profile display with:
  - User icon and name
  - Statistics (observations, identifications, species, journal posts, activity)
  - Account details (member since, site ID, roles)

**Status**: UI implemented, backend integration needed

**Next Steps**:
1. Connect to the iNaturalist API users endpoint
2. Display user icon images
3. Add link to user's observations
4. Add user activity timeline

### 🌿 Taxa
A taxonomic exploration tool for browsing the tree of life.

**Current Features**:
- Taxon search with rank filtering
- Search results sidebar
- Detailed taxon view with:
  - Scientific and common names
  - Observation counts
  - Taxonomic hierarchy/ancestry
  - Wikipedia summaries (placeholder)

**Status**: UI implemented, backend integration needed

**Next Steps**:
1. Connect to the iNaturalist API taxa endpoint
2. Implement real taxonomic hierarchy display
3. Add taxon photos
4. Include Wikipedia integration
5. Add "explore children" functionality

## Architecture

### View-Based Structure

The application uses a view-based architecture where each section is encapsulated in its own view module:

```
src/
├── app.rs              # Main application logic and navigation
├── views/
│   ├── mod.rs          # View module exports
│   ├── identify_view.rs      # Identification interface
│   ├── observations_view.rs  # Observation query interface
│   ├── users_view.rs         # User profile lookup
│   └── taxa_view.rs          # Taxonomic explorer
├── panels/             # Reusable UI panels
└── actors/             # Background data fetching actors
```

### State Management

The application maintains state through:

1. **AppState**: Core application state including:
   - Current view selection
   - Loaded observations (for Identify view)
   - Taxa store
   - Current observation ID

2. **View-Specific State**: Each view maintains its own state:
   - Search queries
   - Filter settings
   - Results
   - Loading states

### Message Passing

The application uses Actix actors for asynchronous data fetching. Messages are passed through unbounded channels:

```rust
AppMessage enum:
- Progress
- TaxonLoaded
- ObservationLoaded
- ComputerVisionScoreLoaded
- TaxonTree
- SkipCurrentObservation
```

## Implementation Guide

### Adding a New View

To add a new view to the application:

1. Create a new view file in `src/views/`:
   ```rust
   pub struct MyNewView {
       // View state
   }
   
   impl MyNewView {
       pub fn show(&mut self, ctx: &egui::Context) {
           // Render UI
       }
   }
   ```

2. Add the view to `src/views/mod.rs`:
   ```rust
   pub mod my_new_view;
   pub use my_new_view::MyNewView;
   ```

3. Add a variant to the `AppView` enum in `src/app.rs`:
   ```rust
   pub enum AppView {
       Identify,
       Observations,
       Users,
       Taxa,
       MyNewView, // Add this
   }
   ```

4. Add the view to `AppViews` struct:
   ```rust
   struct AppViews {
       // ...
       my_new_view: MyNewView,
   }
   ```

5. Add navigation button in `render_ui()`:
   ```rust
   ui.selectable_value(
       &mut self.state.current_view,
       AppView::MyNewView,
       "🎯 My View"
   );
   ```

6. Add render function:
   ```rust
   fn render_my_new_view(&mut self, ctx: &egui::Context) {
       self.views.my_new_view.show(ctx);
   }
   ```

7. Add match arm in `render_ui()`:
   ```rust
   match self.state.current_view {
       // ...
       AppView::MyNewView => self.render_my_new_view(ctx),
   }
   ```

### Connecting to the API

Each view that needs to fetch data should:

1. Send messages to appropriate actors
2. Listen for responses via the message channel
3. Update view state when data arrives

Example:
```rust
// In your view
self.tx_app_message.send(AppMessage::FetchData(params))?;

// In App::process_messages()
match message {
    AppMessage::DataLoaded(data) => {
        // Update view state
    }
}
```

## Design Principles

1. **Separation of Concerns**: Each view is self-contained and manages its own state
2. **Reusability**: Common UI elements are extracted into panels
3. **Async-First**: Data fetching happens in background actors
4. **Progressive Enhancement**: Views show placeholder content until data is available

## Future Enhancements

### Short Term
- Connect all views to the iNaturalist API
- Add image loading and caching
- Implement pagination for large result sets
- Add error handling and user feedback

### Medium Term
- Add user authentication and identification submission
- Implement observation map view
- Add favorites/bookmarking system
- Create project exploration view

### Long Term
- Offline mode with local database
- Advanced filtering and sorting
- Data export functionality
- Integration with other biodiversity platforms

## References

- [iNaturalist API Documentation](https://api.inaturalist.org/v1/docs/)
- [egui Documentation](https://docs.rs/egui/)
- [Actix Documentation](https://actix.rs/docs/)