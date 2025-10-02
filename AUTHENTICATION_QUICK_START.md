# iNaturalist Pro - Authentication Quick Start

## 🎉 What's New?

**You can now use iNaturalist Pro without logging in!**

The app launches immediately and lets you explore observations, taxa, and users without authentication. Log in only when you need to add identifications or use other authenticated features.

---

## 🚀 Getting Started

### Option 1: Use Without Login (Most Features Available)

1. **Launch the app** - it opens instantly, no waiting!
   ```bash
   cargo run --release
   ```

2. **Start exploring:**
   - Browse observations
   - View photos and details
   - Check computer vision suggestions
   - Explore taxonomic trees
   - Search for users

3. **Look for the 🔒 Login button** in the top-right corner - that's how you know you can authenticate when ready.

### Option 2: Log In to Access All Features

1. **Click the 🔒 Login button** in the top-right corner

2. **A modal appears** - click "Login" to proceed

3. **Your browser opens** - authenticate with your iNaturalist account

4. **Return to the app** - you'll now see "👤 Profile" instead of "🔒 Login"

5. **You're authenticated!** - all features are now available

---

## 🔍 Visual Guide

### Not Logged In
```
┌─────────────────────────────────────────────────────────┐
│ File                                          🔒 Login   │
└─────────────────────────────────────────────────────────┘
```

### Logged In
```
┌─────────────────────────────────────────────────────────┐
│ File                                      👤 Profile ▼   │
└─────────────────────────────────────────────────────────┘
```

---

## ✅ What Works Without Login

- 🔍 Browse observations
- 📷 View photos and observation details
- 🤖 See computer vision ID suggestions
- 🌿 Explore taxa and taxonomy
- 👤 Search and view user profiles
- 🗺️ View observation locations

## 🔒 What Requires Login

- ➕ Add identifications to observations
- 📝 Submit new observations (future)
- 👤 Access your personal data (future)
- ⭐ Favorite observations (future)

---

## 💡 Key Points

1. **Token Auto-Saved**: When you log in, your token is saved automatically
2. **Auto-Load Next Time**: If your token is still valid, you'll be logged in automatically on next launch
3. **Token Expiration**: If your token expires, just click "🔒 Login" again
4. **No Forced Login**: Never required to log in just to explore the app

---

## 🐛 Quick Troubleshooting

**Login button not responding?**
- Check if a browser window opened (might be in the background)
- Ensure you have internet connectivity
- Try closing and reopening the app

**"Authentication failed" message?**
- Click "🔒 Login" to try again
- Check your internet connection
- Verify your iNaturalist account is active

**Don't see the login button?**
- You might already be logged in! Look for "👤 Profile" in the top-right
- If you see "👤 Profile", you're already authenticated

**Want to use a different account?**
- Logout feature coming soon!
- For now, delete the config file and restart:
  - macOS/Linux: `~/.config/inaturalist-identifier-pro/`
  - Windows: `%APPDATA%\inaturalist-identifier-pro\`

---

## 📚 More Information

- **Full User Guide**: See `USER_GUIDE.md` for detailed instructions
- **Troubleshooting**: See `TROUBLESHOOTING.md` for common issues
- **Technical Details**: See `AUTHENTICATION_CHANGELOG.md` for implementation details

---

## 🎯 Quick Start Checklist

- [ ] Launch the app (no login required!)
- [ ] Explore observations and taxa
- [ ] When ready, click "🔒 Login" for authenticated features
- [ ] Check for "👤 Profile" to confirm you're logged in
- [ ] Enjoy full access to iNaturalist Pro!

**That's it! You're ready to go.** 🚀