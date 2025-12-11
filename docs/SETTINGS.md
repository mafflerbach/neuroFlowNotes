# Settings

NeuroFlow Notes settings are accessible via the gear icon in the top-right toolbar. Settings are organized into sections using a vertical navigation menu.

---

## Appearance

Customize the visual appearance of the application.

### Theme Mode

Choose how the app determines light or dark appearance:

| Option | Description |
|--------|-------------|
| **System** | Automatically match your operating system's theme preference |
| **Light** | Always use light mode |
| **Dark** | Always use dark mode |

### Color Scheme

Select a color palette for the UI. All schemes are from the [Catppuccin](https://catppuccin.com/) family:

| Scheme | Description |
|--------|-------------|
| **Latte** | Light, warm palette with soft colors |
| **Frappe** | Medium contrast dark theme |
| **Macchiato** | Darker theme with rich colors |
| **Mocha** | Highest contrast dark theme |

Each scheme includes carefully chosen colors for:
- Text and backgrounds
- Syntax highlighting
- UI accents and buttons
- Calendar blocks and tags

---

## Block Colors

Customize the colors of schedule blocks on your calendar. This helps visually distinguish different types of events.

### Available Block Types

| Type | Default Use |
|------|-------------|
| **Default** | Standard blocks without a specific type |
| **Meeting** | Meetings and calls |
| **Focus** | Deep work and focused time |
| **Break** | Breaks and rest periods |
| **Personal** | Personal appointments |
| **Travel** | Travel and commute time |

### Customizing Colors

1. Go to Settings > Block Colors
2. Click on any block type to expand the color picker
3. Select a color using the visual picker or enter a hex code
4. The preview updates in real-time
5. Click outside to close the picker - changes save automatically

### Color Picker Features

- **Visual picker**: Click and drag to select hue and saturation
- **Brightness slider**: Adjust lightness
- **Hex input**: Enter exact color codes (e.g., `#8839ef`)
- **Preview**: See the color applied to a sample block

### Reset Colors

Click **Reset to Defaults** to restore the original Catppuccin-based color scheme.

---

## Properties

Bulk manage property keys and values across your entire vault. This is useful for:

- Fixing typos in property names
- Normalizing casing (e.g., "Project" vs "project")
- Merging duplicate properties
- Cleaning up unused properties

### Three-Column Interface

| Column | Shows |
|--------|-------|
| **Keys** | All property keys with usage counts |
| **Values** | Values for the selected key |
| **Notes** | Notes using the selected key/value |

### Operations

#### Rename Key

1. Select a key
2. Click the pencil icon
3. Enter the new name
4. Press Enter

All notes with that key are updated automatically.

#### Rename Value

1. Select a key
2. Select a value in the Values column
3. Click the pencil icon
4. Enter the new value
5. Press Enter

All notes with that key-value pair are updated.

#### Merge Keys

Combine two property keys into one:

1. Click **Merge** button
2. Click the source key (will be deleted)
3. Click the target key (will remain)
4. Confirm the merge

All values from the source are moved to the target.

#### Delete Key

Remove a property from all notes:

1. Select a key
2. Click the X icon
3. Confirm deletion

### Viewing Usage

- **Usage counts** appear next to each key and value
- **Notes column** shows exactly which notes use the selected property
- Click "All values" to see all notes with any value for that key
- Select a specific value to filter to just notes with that value

See [Properties Documentation](PROPERTIES.md) for more details on working with properties.

---

## Settings Storage

Settings are stored locally and persist between sessions:

- **Location**: Application data directory (platform-specific)
- **Format**: JSON
- **Scope**: Per-user, not per-vault

Settings that are vault-specific (if implemented in the future) would be stored in the vault folder.

---

## Keyboard Shortcuts

There's no dedicated keyboard shortcut to open Settings. Use the gear icon in the toolbar.

Future versions may include:
- `Cmd/Ctrl + ,` to open Settings
- Keyboard navigation within Settings panels

---

## Troubleshooting

### Settings Not Saving

1. Ensure you have write permissions to the app data directory
2. Check if the app closed cleanly (force-quit may not save)
3. Try resetting a specific setting and re-applying

### Theme Not Applying

1. Close and reopen any open notes
2. Try toggling theme mode to System and back
3. Restart the application

### Colors Look Wrong

1. Check your display's color profile settings
2. Try a different color scheme to compare
3. Reset to defaults if customizations look incorrect
