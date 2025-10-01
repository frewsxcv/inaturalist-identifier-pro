# iNaturalist Pro - User Guide

Welcome to iNaturalist Pro! This guide will help you get started with exploring and identifying observations from iNaturalist.

## Table of Contents

1. [Getting Started](#getting-started)
2. [Navigation](#navigation)
3. [Identify View](#identify-view)
4. [Observations View](#observations-view)
5. [Users View](#users-view)
6. [Taxa View](#taxa-view)
7. [Tips & Best Practices](#tips--best-practices)
8. [FAQ](#faq)

---

## Getting Started

### First Launch

When you first launch iNaturalist Pro:

1. **Authentication**: The app will automatically authenticate with iNaturalist using OAuth. You'll see your access token in the logs.

2. **Main Window**: The application window will open with the navigation sidebar on the left.

3. **Choose a View**: Click on any of the four main sections to get started.

### System Requirements

- **Internet Connection**: Required for fetching data from iNaturalist
- **Operating System**: macOS, Linux, or Windows
- **Screen Resolution**: Minimum 1024x768 recommended

---

## Navigation

The application features a simple navigation sidebar on the left side of the window:

```
┌─────────────────┐
│ iNaturalist Pro │
├─────────────────┤
│ 🔍 Identify     │
│ 📷 Observations │
│ 👤 Users        │
│ 🌿 Taxa         │
└─────────────────┘
```

**How to Navigate:**
- Click on any section name to switch views
- The currently selected view is highlighted
- Your position within each view is preserved when you switch

---

## Identify View

The Identify view is designed to help you browse and identify unidentified observations from iNaturalist.

### Getting Started with Identify

1. **Click on "🔍 Identify"** in the navigation sidebar

2. **Start Loading Observations**:
   - You'll see a welcome screen with a "▶ Start Loading Observations" button
   - Click this button to begin fetching observations
   - **Note**: This may take 30 seconds to several minutes depending on how many observations are in the region

3. **Monitor Progress**:
   - A loading spinner will appear
   - You'll see "Loading observations... (X regions loaded)"
   - Wait until observations appear

### Using the Identify Interface

Once observations are loaded, the interface has three main areas:

#### 1. Observation Gallery (Bottom)
- Shows thumbnail images of all loaded observations
- Click on any thumbnail to select that observation
- Observations are sorted by computer vision confidence score (highest first)

#### 2. Main Image & Details (Center-Right)
- Large view of the selected observation's photo
- Observer name and date
- Location information
- Current identifications (if any)

#### 3. Identification Panel (Left)
- **Computer Vision Scores**: AI-suggested identifications with confidence percentages
- **Taxonomic Tree**: Shows the taxonomic hierarchy of suggestions
- **Identification Actions**: Add your own identifications (when implemented)

### Working Through Observations

**Best Practices:**
- Start with observations that have high CV scores (>90%) - these are usually reliable
- Compare the photo with the suggested species
- Use external resources (field guides, other websites) to verify
- Skip observations you're not confident about

**Navigation:**
- Use the gallery to jump between observations
- Click "Skip" to remove an observation you can't identify
- The next highest-scoring observation will be selected automatically

---

## Observations View

The Observations view allows you to query and browse iNaturalist observations with custom filters.

### Using the Search Form

Located at the top of the view, the search form includes:

**Search Fields:**
- **Search**: General text search (species name, description, etc.)
- **Taxon**: Filter by specific taxon (e.g., "Aves" for birds)
- **User**: Show observations by a specific user
- **Place**: Filter by location name
- **Date Range**: 
  - "Date from": Start date (YYYY-MM-DD format)
  - "Date to": End date (YYYY-MM-DD format)

**Filter Options:**
- **Quality Grade**: 
  - Any: All observations
  - Research: Observations with community ID agreement
  - Needs ID: Observations needing more identifications
  - Casual: Observations that don't meet research criteria
  
- **Identified**:
  - Any: All observations
  - Yes: Observations with identifications
  - No: Observations without identifications

**Buttons:**
- **🔍 Search**: Execute the search with current filters
- **Clear**: Reset all filters and results

### Viewing Results

After searching, results appear in a grid format:

| ID | Taxon | User | Date | Place |
|----|-------|------|------|-------|
| ... | ... | ... | ... | ... |

**Status**: 🚧 This view is currently under development. The UI is complete but needs to be connected to the iNaturalist API.

---

## Users View

Look up iNaturalist users and view their profiles and statistics.

### Searching for a User

1. **Enter a username** in the search field
2. **Click "🔍 Search"** to look up the user
3. **View the profile** that appears below

### User Profile Display

When a user is found, you'll see:

**Header:**
- User icon (profile picture)
- Username
- Real name (if provided)
- User ID number

**Statistics:**
- 📷 **Observations**: Total number of observations posted
- 🔍 **Identifications**: Total identifications made for others
- 🌿 **Species**: Number of unique species observed
- 📝 **Journal Posts**: Number of journal entries
- 💫 **Total Activity**: Combined activity count

**Details:**
- **Member since**: Account creation date
- **Site ID**: iNaturalist site identifier
- **Roles**: Any special roles (curator, admin, etc.)

**Status**: 🚧 This view shows example data. API integration is needed to fetch real user information.

---

## Taxa View

Explore the tree of life and search for taxonomic information about any organism.

### Interface Layout

The Taxa view has a two-panel layout:

**Left Panel: Search**
- Search form
- Results list

**Right Panel: Details**
- Selected taxon information
- Taxonomic hierarchy
- Description

### Searching for Taxa

1. **Enter a taxon name** (scientific or common name)
   - Examples: "Quercus", "Oak", "Canis lupus", "Wolf"

2. **Optional: Filter by rank**
   - Kingdom, Phylum, Class, Order, Family, Genus, Species

3. **Click "🔍 Search"**

4. **Click on a result** to view full details

### Viewing Taxon Details

When you select a taxon, the right panel shows:

**Header:**
- Scientific name
- Rank (e.g., Species, Genus, Family)
- Common name (if available)
- Taxon ID

**Statistics:**
- 📷 Observations: Total observations on iNaturalist
- 🏷️ Iconic Taxon: Broad category (Plants, Animals, Fungi, etc.)

**Taxonomy:**
- Complete taxonomic hierarchy from Kingdom to current rank
- Each level shows rank and scientific name
- Current taxon is highlighted

**About:**
- Wikipedia summary (when available)
- Additional taxonomic information

**Status**: 🚧 This view shows example data. API integration is needed to fetch real taxonomic information.

---

## Tips & Best Practices

### For Identifying Observations

1. **Start with your area of expertise**: Focus on taxa you know well
2. **Use the CV scores as a guide**: High scores (>90%) are usually accurate, but always verify
3. **Consider the location**: Is this species found in this area?
4. **Look at details**: Zoom in on key identification features
5. **Check the date**: Is this the right season for this species?
6. **When in doubt, identify to a higher rank**: Better to ID as "Family" correctly than "Species" incorrectly

### For Searching Observations

1. **Start broad, then narrow**: Begin with general filters, add more as needed
2. **Use date ranges**: Filter by season to find specific phenomena
3. **Combine filters**: Use multiple filters together for precise results
4. **Check quality grade**: Research grade observations are more reliable

### For Looking Up Taxa

1. **Try both common and scientific names**: Both work
2. **Start with higher ranks**: Search for Family or Order first, then narrow down
3. **Use rank filters**: Helpful when you know approximately what level to search
4. **Read the taxonomy**: Understanding the hierarchy helps with identification

### Performance Tips

1. **Be patient with loading**: Initial observation loading can take time
2. **Start small**: Test with a smaller region first (modify in settings)
3. **Use specific filters**: Narrower searches are faster
4. **Monitor your internet connection**: The app requires stable connectivity

---

## FAQ

### General Questions

**Q: How do I change the location for the Identify view?**
A: Currently, this requires modifying the source code in `src/places.rs`. A UI for this is planned for a future release.

**Q: Can I use this offline?**
A: No, the app requires an internet connection to fetch data from iNaturalist.

**Q: Is my iNaturalist account needed?**
A: The app authenticates via OAuth, but for browsing public data, no account is strictly required. Adding identifications will require authentication.

**Q: Where is my data stored?**
A: OAuth tokens are stored in your system's config directory. No observation data is stored locally.

### Identify View

**Q: Why do I need to click "Start Loading Observations"?**
A: This prevents automatic loading on startup, which could cause errors or consume bandwidth unexpectedly. You control when to fetch data.

**Q: How many observations will be loaded?**
A: The default limit is 30 observations. This can be configured in the source code.

**Q: Can I load observations from a different location?**
A: Currently, the location is set to New York City by default. You can modify `src/places.rs` and `src/main.rs` to change this.

**Q: Why are some observations missing photos?**
A: iNaturalist observations don't always have photos (some are sound recordings or other media).

**Q: How do I submit an identification?**
A: This feature is planned but not yet implemented in the current version.

### Observations View

**Q: Why don't I see any results?**
A: This view's API integration is still in development. The UI is ready, but it will show placeholder data until the backend is connected.

**Q: What date format should I use?**
A: Use YYYY-MM-DD format (e.g., 2024-01-15).

**Q: Can I export search results?**
A: Not yet, but this feature is planned for a future release.

### Users View

**Q: Why does it always show example data?**
A: The API integration for this view is still in development. The UI shows placeholder data to demonstrate the layout.

**Q: Can I view a user's observations from here?**
A: Not yet, but this feature is planned.

### Taxa View

**Q: Why can't I find a taxon?**
A: The API integration is still in development. Currently, only example data is shown.

**Q: Will there be photos of taxa?**
A: Yes, this is planned for when the API is integrated.

**Q: Can I browse the full tree of life?**
A: Not yet, but exploring taxonomic relationships is a planned feature.

### Technical Questions

**Q: Why is the app slow to start?**
A: The app needs to initialize actors and authenticate with iNaturalist. Subsequent launches should be faster as tokens are cached.

**Q: What do I do if the app crashes?**
A: Check the TROUBLESHOOTING.md file for common issues and solutions. Run with `RUST_LOG=debug` to see detailed logs.

**Q: Can I contribute to development?**
A: Yes! This is an open-source project. Check the repository for contribution guidelines.

**Q: How do I report a bug?**
A: Create an issue on the project's repository with:
- Description of the problem
- Steps to reproduce
- Error messages (if any)
- Your system information

---

## Keyboard Shortcuts

*Currently, there are no keyboard shortcuts implemented. This is a planned feature.*

---

## Getting Help

If you need additional help:

1. Check the **TROUBLESHOOTING.md** file for common issues
2. Review the **NAVIGATION.md** file for architecture details
3. Check the logs with `RUST_LOG=debug cargo run`
4. Create an issue on the project repository

---

## What's Next?

The iNaturalist Pro application is under active development. Upcoming features include:

- ✅ Manual observation loading (implemented)
- ✅ Multi-view navigation (implemented)
- 🚧 API integration for Observations view (in progress)
- 🚧 API integration for Users view (in progress)
- 🚧 API integration for Taxa view (in progress)
- 📋 Image loading and caching (planned)
- 📋 Identification submission (planned)
- 📋 User authentication (planned)
- 📋 Customizable search locations (planned)
- 📋 Export functionality (planned)
- 📋 Keyboard shortcuts (planned)

Thank you for using iNaturalist Pro!