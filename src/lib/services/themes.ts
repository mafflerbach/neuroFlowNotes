/**
 * Theme service - dynamically loads themes from the themes directory
 */

// Use Vite's import.meta.glob to find all theme files
const themeModules = import.meta.glob("../styles/themes/*.css", {
  query: "?inline",
  eager: true,
});

export interface ThemeInfo {
  id: string;
  name: string;
}

/**
 * Get list of available themes
 */
export function getAvailableThemes(): ThemeInfo[] {
  const themes: ThemeInfo[] = [
    { id: "system", name: "System" },
  ];

  for (const path of Object.keys(themeModules)) {
    // Extract filename without extension: "../styles/themes/dark.css" -> "dark"
    const match = path.match(/\/([^/]+)\.css$/);
    if (match) {
      const id = match[1];
      // Capitalize first letter for display name
      const name = id.charAt(0).toUpperCase() + id.slice(1);
      themes.push({ id, name });
    }
  }

  return themes;
}

/**
 * Check if a theme ID is valid
 */
export function isValidTheme(themeId: string): boolean {
  const themes = getAvailableThemes();
  return themes.some((t) => t.id === themeId);
}
