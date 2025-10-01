# Troubleshooting Guide for iNaturalist Pro

## Common Issues and Solutions

### Startup Error: "EOF while parsing a value"

**Error Message:**
```
2025-10-01T12:33:01.826392Z  INFO inaturalist_oauth: OAuth access token: Z4T-fbCRnChuZe8d_PLjJdoRnTwJYVrOlktRMcY7IbI
Error: Error("EOF while parsing a value", line: 1, column: 0)
```

**Cause:**
This error occurred because the application was attempting to fetch observations from the iNaturalist API immediately on startup, before the UI was even displayed. The API call was failing (likely due to network issues, rate limiting, or invalid parameters) and returning an empty or malformed response that couldn't be parsed as JSON.

**Solution:**
The observation loading process has been changed from automatic to manual. Now, observations are only loaded when the user explicitly clicks the "Start Loading Observations" button in the Identify view.

**What Changed:**
1. The `ObservationLoaderActor` no longer starts fetching data automatically when the actor is created
2. Instead, it waits for a `StartLoadingMessage` to be sent to it
3. The Identify view shows a loading screen with a start button when no observations are loaded
4. Better error handling was added to prevent crashes when API calls fail

**Code Changes:**
- Modified `observation_loader_actor.rs` to use a message handler instead of `started()` lifecycle method
- Updated `identify_view.rs` to show a loading screen with a manual start button
- Added error handling to gracefully handle failed API requests
- Stored the observation loader actor address in the app state for access from views

---

## General Troubleshooting Steps

### Application Won't Start

1. **Check your internet connection**
   - The app requires internet access to fetch data from iNaturalist

2. **Verify OAuth token**
   - The app uses OAuth for authentication
   - If you see authentication errors, the app may need to re-authenticate
   - Token is stored in your system's config directory

3. **Check logs**
   - The app uses `tracing` for logging
   - Look for error messages in the terminal output
   - Set `RUST_LOG=debug` environment variable for more detailed logs:
     ```bash
     RUST_LOG=debug cargo run --release
     ```

### API Rate Limiting

**Symptoms:**
- Slow loading
- Missing observations
- API errors in logs

**Solution:**
The app has built-in rate limiting (1 request per 2 seconds), but if you're making many requests, you may hit iNaturalist's API limits.

**What to do:**
1. Wait a few minutes before trying again
2. Reduce the search area (modify the default location in `places.rs`)
3. Use more specific filters to reduce result set size

### No Observations Loading in Identify View

**Symptoms:**
- Loading spinner shows but no observations appear
- Loading counter stays at 0

**Possible Causes:**
1. **No unidentified observations in the search area**
   - Try changing the location in `src/places.rs`
   - Modify the `grid` initialization in `main.rs`

2. **API query parameters too restrictive**
   - Check `operations.rs` to see what filters are being applied
   - The default operation may be filtering out observations

3. **Network issues**
   - Check your internet connection
   - Check if api.inaturalist.org is accessible

**Solution:**
1. Check the logs for error messages
2. Try a different location with known observations
3. Verify the API query parameters

### View Not Displaying Correctly

**Symptoms:**
- Empty panels
- Layout issues
- Missing UI elements

**Solutions:**
1. Resize the window - some panels may be collapsed
2. Switch to a different view and back
3. Restart the application
4. Check terminal for UI-related errors

### Build Errors

**Common Issues:**

1. **Missing dependencies**
   ```bash
   cargo clean
   cargo build --release
   ```

2. **Outdated Rust version**
   ```bash
   rustup update
   ```

3. **Dependency version conflicts**
   - Check `Cargo.toml` files in workspace
   - Update dependencies: `cargo update`

### Runtime Panics

If the application crashes with a panic:

1. Check the panic message for clues
2. Look at the backtrace (set `RUST_BACKTRACE=1`)
3. Check if it's related to:
   - Actor communication (message channel errors)
   - API responses (JSON parsing)
   - UI rendering (egui errors)

**Most Common Panics:**

1. **Actor message send failed**
   - Actor may have stopped
   - Restart the application

2. **JSON parsing error**
   - API returned unexpected format
   - Check API documentation for changes
   - Verify network response

3. **Unwrap on None**
   - Defensive programming issue
   - Report as a bug

---

## Configuration

### Changing the Default Location

By default, the app loads observations from New York City. To change this:

1. Edit `src/places.rs` and add your location:
   ```rust
   static MY_LOCATION_CELL: OnceLock<Rect> = OnceLock::new();
   pub fn my_location() -> &'static Rect {
       MY_LOCATION_CELL.get_or_init(|| {
           geo::Rect::new(
               geo::coord! {
                   x: ordered_float::OrderedFloat(-122.5),
                   y: ordered_float::OrderedFloat(37.5),
               },
               geo::coord! {
                   x: ordered_float::OrderedFloat(-122.0),
                   y: ordered_float::OrderedFloat(38.0),
               },
           )
       })
   }
   ```

2. Update `main.rs` to use your location:
   ```rust
   let grid = GeohashGrid::from_rect(places::my_location().clone(), 4);
   ```

### Adjusting Rate Limits

To change the API rate limit, modify `inaturalist-fetch/src/lib.rs`:

```rust
pub fn inaturalist_rate_limit_amount() -> &'static governor::Quota {
    INATURALIST_RATE_LIMIT_AMOUNT_CELL
        .get_or_init(|| {
            // Change from 2 seconds to your preferred interval
            governor::Quota::with_period(std::time::Duration::from_secs(2)).unwrap()
        })
}
```

### Changing the Observation Limit

To load more or fewer observations, modify the `fetch_soft_limit()` function in `main.rs`:

```rust
fn fetch_soft_limit() -> &'static sync::atomic::AtomicI32 {
    FETCH_SOFT_LIMIT_CELL.get_or_init(|| {
        // Change from 30 to your preferred limit
        sync::atomic::AtomicI32::new(30)
    })
}
```

---

## Getting Help

If you encounter an issue not covered here:

1. **Check the logs** - Run with `RUST_LOG=debug` for detailed output
2. **Check iNaturalist API status** - Visit https://www.inaturalist.org
3. **Review recent changes** - Check git history for recent modifications
4. **Search existing issues** - Look for similar problems in the issue tracker
5. **Create an issue** - Provide:
   - Error message (full text)
   - Steps to reproduce
   - System information (OS, Rust version)
   - Relevant log output

---

## Performance Tips

### Slow Loading

1. **Reduce search area** - Smaller geographic bounds = faster loading
2. **Increase rate limit delay** - More time between requests = more reliable
3. **Use smaller geohash precision** - Change the `4` in `GeohashGrid::from_rect(..., 4)` to `5` or `6`

### High Memory Usage

1. **Limit observation count** - Reduce `fetch_soft_limit()`
2. **Clear results periodically** - Restart the app when memory gets high
3. **Use more specific queries** - Fewer observations = less memory

### UI Lag

1. **Reduce image loading** - Disable or limit concurrent image loads
2. **Simplify UI** - Collapse unused panels
3. **Use release build** - Debug builds are much slower

---

## Developer Notes

### Architecture Overview

The app uses an actor-based architecture:
- **Actors** run in background threads and fetch data asynchronously
- **Message channels** communicate between actors and the UI
- **Views** render UI and handle user interaction
- **App state** maintains the current state of the application

### Adding Better Error Handling

When modifying code, prefer:
- `Result` types over panics
- `if let` or `match` over `unwrap()`
- Logging errors instead of silently ignoring them
- Graceful degradation over crashes

Example:
```rust
// Bad
let data = fetch_data().await.unwrap();

// Good
match fetch_data().await {
    Ok(data) => process_data(data),
    Err(e) => {
        tracing::error!("Failed to fetch data: {}", e);
        // Show error to user or use fallback
    }
}
```

### Testing API Calls

To test API calls without the UI:

```rust
#[tokio::test]
async fn test_fetch_observations() {
    let api_token = "your_token_here";
    let result = fetch_observations(api_token).await;
    assert!(result.is_ok());
}
```

---

## Version History

### Current Version
- Manual observation loading (fixed startup crash)
- Multi-view navigation
- Better error handling

### Known Issues
- Observation, Users, and Taxa views need API integration
- Image loading not yet implemented for new views
- No pagination in search results