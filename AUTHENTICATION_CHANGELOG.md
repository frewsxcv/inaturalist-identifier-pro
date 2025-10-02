# Authentication System Changelog

## Overview

The authentication system in iNaturalist Pro has been redesigned to provide a better user experience by making login **optional at startup**. Users can now start using the app immediately and authenticate only when needed for specific features.

## Release Date

January 2025

---

## What Changed

### Before (Old Behavior)

- **Blocking Authentication**: The app would block on startup, forcing OAuth authentication before the UI appeared
- **Mandatory Login**: Users had to authenticate even if they only wanted to browse public data
- **Poor Error Handling**: If authentication failed, the app couldn't start
- **No Flexibility**: No way to use the app without an iNaturalist account

### After (New Behavior)

- **Non-blocking Startup**: The app launches immediately without waiting for authentication
- **Optional Login**: Users can browse public data without logging in
- **Dynamic Authentication**: Login button available in the UI when authentication is needed
- **Graceful Token Loading**: Saved tokens are loaded automatically if valid, but startup continues if none exists
- **Better UX**: Profile button/icon shows authentication status in the top-right corner

---

## User-Facing Changes

### New UI Elements

1. **Login Button** (🔒 Login)
   - Located in the top-right corner of the window
   - Visible when user is not authenticated
   - Clicking opens a login modal with OAuth instructions

2. **Profile Menu** (👤 Profile)
   - Replaces login button when user is authenticated
   - Shows "✅ Logged in" status
   - Contains account options (coming soon)
   - Includes logout option (coming soon)

3. **Login Modal**
   - Clean dialog explaining the OAuth process
   - "Login" and "Cancel" buttons
   - Status messages for authentication progress
   - User-friendly error messages

### Features Available Without Login

✅ Browse observations
✅ View observation details and photos
✅ View computer vision suggestions
✅ Explore taxonomic hierarchies
✅ Search for users
✅ View user profiles
✅ Navigate between all app views

### Features Requiring Login

🔒 Adding identifications to observations
🔒 Submitting new observations (future)
🔒 Managing personal data (future)

---

## Technical Changes

### Modified Files

#### `main.rs`
- Removed blocking OAuth call at startup
- Changed token loading to be non-blocking
- Token check now only validates expiration, doesn't require authentication
- Actors initialized with empty string tokens if no authentication
- App receives client credentials for dynamic authentication
- Added `AuthenticationComplete` and `AuthenticationFailed` messages

#### `app.rs`
- Added `api_token: Option<String>` field to App struct
- Added `client_id` and `client_secret` fields for dynamic authentication
- Made `AppState` and its fields public for external access
- Added `is_authenticated`, `show_login_modal`, and `auth_status_message` to AppState
- Implemented `show_login_modal()` method
- Implemented `initiate_login()` method with background authentication
- Refactored message processing to handle auth messages
- Removed MessageHandler struct (simplified inline)
- Updated `process_messages()` to handle authentication state changes

#### `panels/top_panel.rs`
- Complete redesign to support authentication UI
- Added profile/login button to menu bar
- Implemented `show_authenticated_menu()` for logged-in users
- Implemented `show_login_button()` for non-authenticated users
- Right-aligned authentication controls in menu bar
- Status message display for authentication feedback

#### Documentation Updates
- `USER_GUIDE.md`: Added comprehensive Authentication section
- `TROUBLESHOOTING.md`: Added authentication troubleshooting section
- `README.md`: Updated features section with authentication info

### New Message Types

```rust
pub enum AppMessage {
    // ... existing messages ...
    AuthenticationComplete(String),  // Contains API token
    AuthenticationFailed(String),    // Contains error message
}
```

### Actor Changes

- All actors now accept empty strings for `api_token` at initialization
- Actors will use the token if available, or fail gracefully if features require auth
- No breaking changes to actor interfaces

---

## Benefits

### For Users

1. **Faster Startup**: No waiting for OAuth browser windows or authentication
2. **Try Before Login**: Explore the app without creating an account
3. **Better Privacy**: Only authenticate when necessary
4. **Clearer Requirements**: Visual indicators show when login is needed
5. **Persistent Sessions**: Valid tokens automatically loaded on subsequent launches

### For Developers

1. **Better Error Handling**: Authentication failures don't crash the app
2. **More Flexible**: Easier to test without authentication
3. **Cleaner Architecture**: Separation of concerns between auth and app logic
4. **Better UX**: Users see the UI immediately, increasing engagement

---

## Implementation Details

### Token Management

- **Storage Location**: `~/.config/inaturalist-identifier-pro/` (Linux/macOS) or `%APPDATA%\inaturalist-identifier-pro\` (Windows)
- **Format**: Configuration file managed by `confy` crate
- **Security**: Token stored locally, transmitted only to iNaturalist API
- **Lifetime**: Tokens checked for expiration on startup

### Authentication Flow

1. User clicks "🔒 Login" button
2. Login modal appears with instructions
3. User clicks "Login" in modal
4. Background task spawns to handle OAuth:
   - Creates new `Authenticator` instance
   - Calls `get_api_token()` (opens browser)
   - User authenticates in browser
   - Token received and saved to config
5. `AuthenticationComplete` message sent to app
6. UI updates to show "👤 Profile" button
7. App state updated with token and auth status

### Error Handling

- Network errors: Displayed in modal with retry option
- Browser issues: Instructions provided to user
- Token expiration: Automatic re-authentication prompt
- Invalid credentials: Error message with support info

---

## Breaking Changes

**None**. This update is fully backward compatible. Existing saved tokens will continue to work.

---

## Future Enhancements

- [ ] Implement logout functionality
- [ ] Display user account information in Profile menu
- [ ] Add "Remember Me" option
- [ ] Support multiple accounts
- [ ] Token refresh mechanism
- [ ] Authentication status in status bar

---

## Migration Guide

No migration required! The app will automatically:
- Load existing tokens if they're valid
- Continue working if no token exists
- Prompt for login only when needed

Users with existing tokens will see "👤 Profile" immediately on startup, indicating they're already logged in.

---

## Support

For authentication issues, see `TROUBLESHOOTING.md` or open an issue on GitHub.