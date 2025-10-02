# iNaturalist Pro

A Rust native application for exploring and interacting with iNaturalist.

## Features

### 🔒 Authentication
Start using the app immediately without logging in! Browse public observations, explore taxa, and view user profiles. When you need features that require authentication (like adding identifications), simply click the "🔒 Login" button in the top-right corner. Your authentication token is securely saved and automatically loaded on subsequent launches.

### 🔍 Identify
A powerful interface for identifying observations with computer vision scores and taxonomic information. Browse unidentified observations and help the community with identifications.

### 📷 Observations
Query and browse iNaturalist observations by location, taxon, user, and more (coming soon).

### 👤 Users
Search and view iNaturalist user profiles, stats, and observations (coming soon).

### 🌿 Taxa
Explore the tree of life, search taxa, and view detailed taxonomic information (coming soon).

## Screenshot

<img width="1393" height="794" alt="Screenshot of iNaturalist Pro application showing the Identify interface" src="https://github.com/user-attachments/assets/f5461dbd-9d95-4b86-8036-11c5a10dd310" />

## Building

This is a Rust workspace with multiple crates. To build and run:

```bash
cargo run --release
```

## Quick Start

1. Build and run the application:
   ```bash
   cargo run --release
   ```

2. The app will open immediately - no login required!

3. Browse observations, explore taxa, and search users without authentication.

4. When you need to add identifications or access authenticated features, click the "🔒 Login" button in the top-right corner.

## Architecture

- **inaturalist_pro**: Main GUI application built with egui
- **inaturalist-fetch**: Library for fetching data from iNaturalist API
- **geo-ext**: Geographic utilities and extensions

## License

This project is a proof-of-concept application for interacting with the iNaturalist platform.