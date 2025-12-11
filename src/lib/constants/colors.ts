/**
 * Color palette for schedule blocks
 * Uses CSS variables to support theming
 */

export interface BlockColor {
  name: string;
  id: string; // CSS variable suffix (e.g., "blue" for --block-color-blue)
  textColor: string; // For contrast on the block
}

// Block colors using CSS variable IDs
// The actual colors are defined in each theme's CSS file
export const BLOCK_COLORS: BlockColor[] = [
  { name: "Blue", id: "blue", textColor: "var(--block-text)" },
  { name: "Sapphire", id: "sapphire", textColor: "var(--block-text)" },
  { name: "Sky", id: "sky", textColor: "var(--block-text)" },
  { name: "Teal", id: "teal", textColor: "var(--block-text)" },
  { name: "Green", id: "green", textColor: "var(--block-text)" },
  { name: "Yellow", id: "yellow", textColor: "var(--block-text)" },
  { name: "Peach", id: "peach", textColor: "var(--block-text)" },
  { name: "Maroon", id: "maroon", textColor: "var(--block-text)" },
  { name: "Red", id: "red", textColor: "var(--block-text)" },
  { name: "Mauve", id: "mauve", textColor: "var(--block-text)" },
  { name: "Pink", id: "pink", textColor: "var(--block-text)" },
  { name: "Flamingo", id: "flamingo", textColor: "var(--block-text)" },
  { name: "Rosewater", id: "rosewater", textColor: "var(--block-text)" },
  { name: "Lavender", id: "lavender", textColor: "var(--block-text)" },
];

/**
 * Get CSS variable for a block color
 */
export function getBlockColorVar(id: string): string {
  return `var(--block-color-${id})`;
}

// Default color for new blocks
export const DEFAULT_BLOCK_COLOR = BLOCK_COLORS[0]; // Blue

// Get a color by ID
export function getBlockColor(id: string | null | undefined): BlockColor {
  if (!id) return DEFAULT_BLOCK_COLOR;
  return BLOCK_COLORS.find((c) => c.id === id) || DEFAULT_BLOCK_COLOR;
}
