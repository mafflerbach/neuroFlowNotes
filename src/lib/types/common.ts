/**
 * Common types shared across the application.
 */

/** Property type identifiers used throughout the app. */
export type PropertyType = "text" | "date" | "number" | "boolean" | "list";

/** Property type display configuration. */
export interface PropertyTypeConfig {
  value: PropertyType;
  label: string;
}

/** Standard property type configurations. */
export const PROPERTY_TYPES: PropertyTypeConfig[] = [
  { value: "text", label: "Text" },
  { value: "date", label: "Date" },
  { value: "number", label: "Number" },
  { value: "boolean", label: "Boolean" },
  { value: "list", label: "List" },
];

/** Check if a string is a valid PropertyType. */
export function isValidPropertyType(type: string | null | undefined): type is PropertyType {
  if (!type) return false;
  return ["text", "date", "number", "boolean", "list"].includes(type);
}

/** Get default value for a property type. */
export function getDefaultValueForType(type: PropertyType): string {
  switch (type) {
    case "boolean":
      return "false";
    case "number":
      return "0";
    case "date":
      return new Date().toISOString().split("T")[0];
    case "list":
      return "";
    case "text":
    default:
      return "";
  }
}
