/**
 * Color palette for schedule blocks - Catppuccin Mocha
 * https://catppuccin.com/palette/
 */

export interface BlockColor {
  name: string;
  hex: string;
  textColor: string; // For contrast on the block
}

// Catppuccin Mocha accent colors (suitable for schedule blocks)
export const BLOCK_COLORS: BlockColor[] = [
  { name: "Blue", hex: "#89b4fa", textColor: "#1e1e2e" },
  { name: "Sapphire", hex: "#74c7ec", textColor: "#1e1e2e" },
  { name: "Sky", hex: "#89dceb", textColor: "#1e1e2e" },
  { name: "Teal", hex: "#94e2d5", textColor: "#1e1e2e" },
  { name: "Green", hex: "#a6e3a1", textColor: "#1e1e2e" },
  { name: "Yellow", hex: "#f9e2af", textColor: "#1e1e2e" },
  { name: "Peach", hex: "#fab387", textColor: "#1e1e2e" },
  { name: "Maroon", hex: "#eba0ac", textColor: "#1e1e2e" },
  { name: "Red", hex: "#f38ba8", textColor: "#1e1e2e" },
  { name: "Mauve", hex: "#cba6f7", textColor: "#1e1e2e" },
  { name: "Pink", hex: "#f5c2e7", textColor: "#1e1e2e" },
  { name: "Flamingo", hex: "#f2cdcd", textColor: "#1e1e2e" },
  { name: "Rosewater", hex: "#f5e0dc", textColor: "#1e1e2e" },
  { name: "Lavender", hex: "#b4befe", textColor: "#1e1e2e" },
];

// Default color for new blocks
export const DEFAULT_BLOCK_COLOR = BLOCK_COLORS[0]; // Blue

// Get a color by hex value
export function getBlockColor(hex: string | null | undefined): BlockColor {
  if (!hex) return DEFAULT_BLOCK_COLOR;
  return BLOCK_COLORS.find((c) => c.hex.toLowerCase() === hex.toLowerCase()) || DEFAULT_BLOCK_COLOR;
}

// Catppuccin Mocha base colors (for reference/theming)
export const MOCHA = {
  rosewater: "#f5e0dc",
  flamingo: "#f2cdcd",
  pink: "#f5c2e7",
  mauve: "#cba6f7",
  red: "#f38ba8",
  maroon: "#eba0ac",
  peach: "#fab387",
  yellow: "#f9e2af",
  green: "#a6e3a1",
  teal: "#94e2d5",
  sky: "#89dceb",
  sapphire: "#74c7ec",
  blue: "#89b4fa",
  lavender: "#b4befe",
  text: "#cdd6f4",
  subtext1: "#bac2de",
  subtext0: "#a6adc8",
  overlay2: "#9399b2",
  overlay1: "#7f849c",
  overlay0: "#6c7086",
  surface2: "#585b70",
  surface1: "#45475a",
  surface0: "#313244",
  base: "#1e1e2e",
  mantle: "#181825",
  crust: "#11111b",
} as const;
