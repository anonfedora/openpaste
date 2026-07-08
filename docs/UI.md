# OpenPaste UI Design

## Overview

OpenPaste desktop UI is built with React, TypeScript, and Tailwind CSS using Tauri. The design prioritizes keyboard-first navigation, instant search, and minimal visual clutter while maintaining a traditional, professional appearance similar to native desktop applications.

## Application Structure

### Window Layout

```
┌─────────────────────────────────────────────────────────┐
│ OpenPaste                                    [─][□][X] │
├─────────────────────────────────────────────────────────┤
│ [🔍 Search clipboard...              [⚙️] [🔒]        │
├─────────────────────────────────────────────────────────┤
│ Filters: [All] [Text] [Images] [Files] [Pinned]       │
├─────────────────────────────────────────────────────────┤
│ ┌─────────────────────────────────────────────────────┐ │
│ │ 📄 Hello, World!                        Chrome  2m  │ │ │
│ │ Plain text • 12 bytes                              │ │ │
│ ├─────────────────────────────────────────────────────┤ │
│ │ 🖼️ Screenshot.png                    VS Code  5m  │ │ │
│ │ Image • 256KB                                       │ │ │
│ ├─────────────────────────────────────────────────────┤ │
│ │ 📄 https://example.com               Firefox 10m  │ │ │
│ │ URL • https://example.com                          │ │ │
│ └─────────────────────────────────────────────────────┘ │
├─────────────────────────────────────────────────────────┤
│ [Collections] [History] [Settings] [Plugins]          │
└─────────────────────────────────────────────────────────┘
```

### Main Components

1. **Header:** Search bar, settings, lock button
2. **Filter Bar:** Content type filters
3. **Results List:** Clipboard items with previews
4. **Sidebar/Navigation:** Collections, history, settings
5. **Status Bar:** Item count, sync status, encryption status

## Navigation

### Keyboard Shortcuts

**Global Shortcuts:**
- `Ctrl/Cmd+K`: Focus search
- `Ctrl/Cmd+Shift+K`: Open main window
- `Ctrl/Cmd+L`: Lock vault
- `Ctrl/Cmd+,`: Open settings
- `Ctrl/Cmd+Q`: Quit
- `Escape`: Close window / Clear search

**Search Shortcuts:**
- `Arrow Down/Up`: Navigate results
- `Enter`: Copy selected item
- `Ctrl/Cmd+Enter`: Copy and paste
- `Ctrl/Cmd+P`: Pin selected item
- `Ctrl/Cmd+F`: Favorite selected item
- `Ctrl/Cmd+D`: Delete selected item
- `Ctrl/Cmd+E`: Edit selected item
- `Ctrl/Cmd+Shift+C`: Copy preview text

**Navigation Shortcuts:**
- `Ctrl/Cmd+1`: All items
- `Ctrl/Cmd+2`: Text only
- `Ctrl/Cmd+3`: Images only
- `Ctrl/Cmd+4`: Files only
- `Ctrl/Cmd+5`: Pinned items
- `Ctrl/Cmd+6`: Favorites

### Navigation Patterns

**Breadcrumb Navigation:**
```
All Items > Code > Rust
```

**Tab Navigation:**
- Collections
- History
- Settings
- Plugins

**Sidebar Navigation:**
- Tree view for collections
- Nested collections support

## Search Interface

### Search Bar

**Features:**
- Instant search (debounced by 150ms)
- Search suggestions
- Recent searches dropdown
- Clear button (X)
- Search filters dropdown

**Search Syntax:**
```
hello world              # Basic search
"exact phrase"          # Phrase search
type:text               # Filter by type
app:chrome              # Filter by app
tag:important           # Filter by tag
after:2024-01-01        # Date range
before:2024-12-31
hello -world            # Exclude term
```

**Search Results:**
- Highlighted matches
- Relevance score
- Content preview
- Source app
- Timestamp

### Search Filters

**Filter Dropdown:**
- Content type (text, image, file, etc.)
- Date range (today, yesterday, week, month, custom)
- Source app
- Tags
- Collections
- Pinned/Favorite

**Active Filters:**
- Displayed as chips
- Click to remove
- Clear all button

## Collections

### Collection Management

**Create Collection:**
- Button in sidebar
- Dialog with name, color, icon
- Optional description

**Edit Collection:**
- Right-click context menu
- Edit name, color, icon
- Delete collection

**Collection Views:**
- List view (default)
- Grid view (for images)
- Compact view

**Smart Collections:**
- All Items (default)
- Pinned
- Favorites
- Recent (last 7 days)
- Frequent (top 10%)

### Adding to Collections

**Drag and Drop:**
- Drag item to collection
- Visual feedback

**Context Menu:**
- "Add to collection" submenu
- Multi-select support

**Keyboard:**
- Select item(s)
- `Ctrl/Cmd+Shift+A` to add to collection

## Item View

### Item List Item

**Layout:**
```
┌─────────────────────────────────────────────────────┐
│ [Icon] Content preview              [App] [Time]    │
│ Metadata (type, size, tags)                          │
│ [Pin] [Favorite] [Copy] [Delete]                    │
└─────────────────────────────────────────────────────┘
```

**Content Preview:**
- Text: First 200 characters
- Image: Thumbnail (64x64)
- File: Icon + filename
- HTML: Text preview

**Actions:**
- Click to view details
- Double-click to copy
- Right-click for context menu

### Item Detail View

**Full Content Display:**
- Text: Full text with syntax highlighting
- Image: Full image with zoom
- HTML: Rendered HTML
- File: File metadata

**Metadata Panel:**
- Created at
- Last accessed
- Access count
- Source app
- Source window
- Tags
- Collection

**Actions:**
- Copy content
- Copy preview
- Pin/Unpin
- Favorite/Unfavorite
- Edit
- Delete
- Add to collection
- Export

## Settings

### Settings Navigation

**Categories:**
- General
- Clipboard
- Search
- Storage
- Encryption
- Sync
- Appearance
- Keyboard Shortcuts
- Plugins
- Advanced

### General Settings

**Startup:**
- [ ] Launch on startup
- [ ] Launch minimized
- [ ] Show tray icon

**Behavior:**
- [ ] Auto-paste on selection
- [ ] Play sound on copy
- [ ] Show notifications

**Language:** Dropdown selection

### Clipboard Settings

**Capture:**
- [ ] Capture text
- [ ] Capture images
- [ ] Capture HTML
- [ ] Capture files
- [ ] Ignore own copies

**Retention:**
- Max items: [10000]
- Max age: [90] days
- [ ] Auto-delete old items

**Duplicates:**
- [ ] Detect duplicates
- Action: [Skip / Replace / Keep Both]

### Search Settings

**Search Behavior:**
- [ ] Instant search
- Debounce: [150] ms
- [ ] Highlight matches
- [ ] Fuzzy search
- [ ] Remove diacritics

**Search Ranking:**
- Recency boost: [0.5]
- Frequency boost: [0.3]
- Pinned boost: [10.0]
- Favorite boost: [5.0]

### Storage Settings

**Compression:**
- [ ] Enable compression
- Level: [3]

**File Storage:**
- Location: [Browse...]
- Max size: [10] GB

**Backup:**
- [ ] Auto-backup
- Frequency: [Daily]
- Location: [Browse...]

### Encryption Settings

**Status:**
- Encryption: [Enabled/Disabled]
- Vault: [Locked/Unlocked]

**Master Password:**
- Change password
- Password strength indicator

**Options:**
- [ ] Encrypt content
- [ ] Encrypt metadata
- [ ] Encrypt file paths
- Auto-lock after: [5] minutes

### Sync Settings

**Sync Provider:**
- None
- WebDAV
- S3
- Git

**Provider Configuration:**
- URL/Endpoint
- Username/Password
- API Key
- Advanced options

**Sync Behavior:**
- [ ] Auto-sync
- Frequency: [Every 5 minutes]
- [ ] Sync on startup
- [ ] Sync on shutdown

### Appearance Settings

**Theme:**
- Light
- Dark
- System
- Custom

**Accent Color:**
- Color picker
- Preset colors

**Font:**
- UI font: [Inter]
- Code font: [JetBrains Mono]
- Font size: [14] px

**Layout:**
- [ ] Compact mode
- [ ] Show thumbnails
- [ ] Show metadata
- [ ] Show source app

### Keyboard Shortcuts

**Shortcut Editor:**
- List of all shortcuts
- Click to edit
- Reset to defaults
- Export/Import shortcuts

### Plugins

**Plugin List:**
- Installed plugins
- Enable/disable toggle
- Plugin settings
- Remove plugin

**Plugin Store:**
- Browse available plugins
- Install plugin
- View plugin details

### Advanced

**Debug:**
- [ ] Enable debug mode
- [ ] Log to file
- Log level: [Info]

**Performance:**
- Max search results: [50]
- Cache size: [100] MB
- [ ] Enable hardware acceleration

**Data:**
- Export data
- Import data
- Reset all data
- Clear history

## Context Menus

### Item Context Menu

**Actions:**
- Copy
- Copy Preview
- Pin/Unpin
- Favorite/Unfavorite
- Add to Collection > [Submenu]
- Edit
- Delete
- Export
- View Details

### Collection Context Menu

**Actions:**
- Rename
- Change Color
- Change Icon
- Delete
- Create Subcollection

### Global Context Menu

**Tray Icon:**
- Open OpenPaste
- Search
- Recent Items > [Submenu]
- Settings
- Lock
- Quit

## Notifications

### Notification Types

**Clipboard Captured:**
- Toast notification
- Shows preview
- Auto-dismiss after 3 seconds
- Click to view

**Sync Status:**
- Sync started
- Sync completed
- Sync error

**Encryption:**
- Vault locked
- Vault unlocked
- Password changed

**Errors:**
- Error message
- Action button (if applicable)

### Notification Settings

**Enable/Disable:**
- Clipboard notifications
- Sync notifications
- Encryption notifications
- Error notifications

**Duration:**
- Auto-dismiss after: [3] seconds

## Accessibility

### Keyboard Navigation

**Tab Order:**
- Logical tab order
- Skip to content link
- Focus indicators

**Screen Reader:**
- ARIA labels
- Live regions for updates
- Announce search results
- Announce actions

### High Contrast

**Support:**
- High contrast mode
- Respect system settings
- Adjustable contrast levels

### Font Scaling

**Support:**
- Respect system font size
- Adjustable UI scale
- Minimum font size

### Color Blindness

**Considerations:**
- Not just color for indicators
- Patterns/shapes for differentiation
- Colorblind-friendly palette

## Responsive Design

### Window Sizes

**Minimum:** 800x600

**Recommended:** 1200x800

**Resizable:** Yes

### Adaptive Layout

**Narrow Window (< 800px):**
- Hide sidebar
- Collapsible filters
- Compact list view

**Wide Window (> 1400px):**
- Show sidebar
- Full filters
- Grid view option

## Performance

### Performance Targets

- **Search Response:** < 100ms UI update
- **List Rendering:** < 50ms for 100 items
- **Window Open:** < 200ms
- **Animation:** 60fps

### Optimization

**Virtual Scrolling:**
- Render only visible items
- Lazy load images
- Recycle components

**Debouncing:**
- Search input: 150ms
- Resize events: 100ms

**Memoization:**
- Memoize expensive computations
- Cache search results
- Cache component renders

## State Management

### State Architecture

**Global State (Zustand):**
```typescript
interface AppState {
  // Search
  searchQuery: string;
  searchResults: ClipboardItem[];
  searchFilters: SearchFilters;
  
  // UI
  selectedItemId: number | null;
  currentView: View;
  sidebarOpen: boolean;
  
  // Clipboard
  clipboardItems: ClipboardItem[];
  
  // Settings
  settings: Settings;
  
  // Encryption
  encryptionLocked: boolean;
  
  // Sync
  syncStatus: SyncStatus;
}
```

### Data Fetching

**TanStack Query:**
```typescript
const { data: items } = useQuery({
  queryKey: ['clipboard-items'],
  queryFn: fetchClipboardItems,
  staleTime: 30000,
});
```

**Real-time Updates:**
- WebSocket connection
- Invalidate queries on events
- Optimistic updates

## Error Handling

### Error States

**Connection Error:**
- Show error banner
- Retry button
- Offline indicator

**Search Error:**
- Show error message
- Suggest fix
- Retry button

**Encryption Error:**
- Show lock screen
- Error message
- Recovery options

### Loading States

**Skeleton Loading:**
- Skeleton screens
- Shimmer effect
- Progressive loading

**Progress Indicators:**
- Progress bars
- Spinners
- Status text

## Internationalization

### Supported Languages

- English (default)
- Spanish
- French
- German
- Japanese
- Chinese (Simplified)
- Chinese (Traditional)

### i18n Implementation

**Library:** `i18next`

**Translation Files:**
```
locales/
  en.json
  es.json
  fr.json
  de.json
  ja.json
  zh-CN.json
  zh-TW.json
```

**Usage:**
```typescript
const { t } = useTranslation();
t('search.placeholder'); // "Search clipboard..."
```

## Theming

### Theme System

**CSS Variables:**
```css
:root {
  --bg-primary: #ffffff;
  --bg-secondary: #f3f4f6;
  --text-primary: #111827;
  --text-secondary: #6b7280;
  --accent: #3b82f6;
  --border: #e5e7eb;
}

[data-theme="dark"] {
  --bg-primary: #1f2937;
  --bg-secondary: #111827;
  --text-primary: #f9fafb;
  --text-secondary: #9ca3af;
  --accent: #60a5fa;
  --border: #374151;
}
```

### Custom Themes

**Theme Editor:**
- Color picker for each variable
- Preview in real-time
- Save custom theme
- Export/import theme

## Wireframes

### Main Window

**Search View:**
- Search bar at top
- Filter bar below search
- Results list fills remaining space
- Status bar at bottom

**Detail View:**
- Split layout
- Left: Item list
- Right: Item details
- Resizable divider

**Settings View:**
- Sidebar with categories
- Main content area with settings
- Save/Cancel buttons

### Dialogs

**Create Collection:**
- Name input
- Color picker
- Icon selector
- Description textarea
- Cancel/Save buttons

**Change Password:**
- Current password
- New password
- Confirm password
- Strength indicator
- Cancel/Save buttons

**Export:**
- Format selection (JSON, CSV, TXT)
- Date range
- Include options
- Export button

## Animation

### Animations

**Transitions:**
- Fade in/out: 200ms
- Slide in/out: 300ms
- Scale: 200ms

**Micro-interactions:**
- Button hover: 150ms
- List item hover: 100ms
- Focus ring: 200ms

**Loading:**
- Spinner: 1s rotation
- Shimmer: 1.5s
- Pulse: 2s

### Animation Guidelines

**Purpose:**
- Provide feedback
- Guide attention
- Smooth transitions

**Avoid:**
- Distracting animations
- Slow animations
- Unnecessary motion

## User Onboarding

### First Run

**Welcome Screen:**
- Introduction
- Feature overview
- Keyboard shortcuts
- Get started button

**Setup Wizard:**
- Step 1: Preferences
- Step 2: Encryption (optional)
- Step 3: Sync (optional)
- Step 4: Complete

### Tips

**Contextual Tips:**
- Tooltip on first use
- Highlight new features
- Suggest keyboard shortcuts

**Progressive Disclosure:**
- Show advanced options later
- Learn as you use
- Help button always available

## Help System

### Help Content

**Documentation:**
- User guide
- Keyboard shortcuts
- FAQ
- Troubleshooting

**In-App Help:**
- Help button in header
- Context-sensitive help
- Search help

### Support

**Contact:**
- Issue tracker link
- Email support
- Community forum

## Analytics (Optional)

**Opt-in Only:**
- Anonymous usage data
- Feature usage
- Performance metrics
- Error reports

**Privacy:**
- No clipboard content
- No personal data
- Aggregate data only
- Can be disabled
