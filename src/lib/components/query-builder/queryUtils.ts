/**
 * Query builder utilities - operator labels, type mappings, and YAML generation.
 */

import type {
  PropertyFilter,
  PropertyOperator,
  FilterMatchMode,
  QueryResultType,
  QueryViewType,
  PropertyKeyInfo,
} from "../../types";

// ============================================================================
// Operator Labels
// ============================================================================

/** All operator labels. */
export const OPERATOR_LABELS: Record<PropertyOperator, string> = {
  Exists: "exists",
  NotExists: "does not exist",
  Equals: "equals",
  NotEquals: "does not equal",
  Contains: "contains",
  StartsWith: "starts with",
  EndsWith: "ends with",
  ContainsAll: "contains all",
  ContainsAny: "contains any",
  DateOn: "on date",
  DateBefore: "before",
  DateAfter: "after",
  DateOnOrBefore: "on or before",
  DateOnOrAfter: "on or after",
};

/** Path filter operators (subset that make sense for paths). */
export const PATH_OPERATOR_LABELS: Record<string, string> = {
  StartsWith: "in folder",
  Contains: "contains",
  Equals: "equals",
  NotEquals: "does not equal",
};

/** Tags filter operators. */
export const TAGS_OPERATOR_LABELS: Record<string, string> = {
  Exists: "has any tags",
  NotExists: "has no tags",
  Equals: "has tag",
  ContainsAll: "has all tags",
  ContainsAny: "has any of tags",
};

/** Date type operators. */
export const DATE_TYPE_OPERATORS: Record<string, string> = {
  Exists: "exists",
  NotExists: "does not exist",
  DateOn: "on date",
  DateBefore: "before",
  DateAfter: "after",
  DateOnOrBefore: "on or before",
  DateOnOrAfter: "on or after",
};

/** Number type operators. */
export const NUMBER_OPERATORS: Record<string, string> = {
  Exists: "exists",
  NotExists: "does not exist",
  Equals: "equals",
  NotEquals: "does not equal",
};

/** Boolean type operators. */
export const BOOLEAN_OPERATORS: Record<string, string> = {
  Exists: "exists",
  NotExists: "does not exist",
  Equals: "is",
};

/** List type operators. */
export const LIST_OPERATORS: Record<string, string> = {
  Exists: "exists",
  NotExists: "does not exist",
  Contains: "contains",
  ContainsAll: "contains all",
  ContainsAny: "contains any",
};

// ============================================================================
// Operator Categories
// ============================================================================

/** Operators that don't need a value. */
export const VALUELESS_OPERATORS: PropertyOperator[] = ["Exists", "NotExists"];

/** Operators that accept multiple values (comma-separated). */
export const MULTI_VALUE_OPERATORS: PropertyOperator[] = ["ContainsAll", "ContainsAny"];

/** Date comparison operators. */
export const DATE_OPERATORS: PropertyOperator[] = [
  "DateOn",
  "DateBefore",
  "DateAfter",
  "DateOnOrBefore",
  "DateOnOrAfter",
];

// ============================================================================
// Special Keys
// ============================================================================

/** Special built-in filter keys. */
export const SPECIAL_KEYS = [
  { key: "_path", label: "Folder (path)", isSpecial: true },
  { key: "_tags", label: "Tags", isSpecial: true },
];

/** Check if a key is a special built-in key. */
export function isSpecialKey(key: string): boolean {
  return key.startsWith("_");
}

// ============================================================================
// Property Type Helpers
// ============================================================================

/** Get property type for a key. */
export function getPropertyType(key: string, propertyKeys: PropertyKeyInfo[]): string | null {
  const propInfo = propertyKeys.find((p) => p.key === key);
  return propInfo?.property_type ?? null;
}

/** Get operator labels for a given key (based on property type). */
export function getOperatorLabels(key: string, propertyKeys: PropertyKeyInfo[]): Record<string, string> {
  if (key === "_path") {
    return PATH_OPERATOR_LABELS;
  }
  if (key === "_tags") {
    return TAGS_OPERATOR_LABELS;
  }

  const propType = getPropertyType(key, propertyKeys);
  switch (propType) {
    case "date":
      return DATE_TYPE_OPERATORS;
    case "number":
      return NUMBER_OPERATORS;
    case "boolean":
      return BOOLEAN_OPERATORS;
    case "list":
      return LIST_OPERATORS;
    case "text":
    default:
      return OPERATOR_LABELS;
  }
}

// ============================================================================
// YAML Generation
// ============================================================================

export interface YamlGeneratorOptions {
  filters: PropertyFilter[];
  matchMode: FilterMatchMode;
  resultType: QueryResultType;
  includeCompleted: boolean;
  viewType: QueryViewType;
  kanbanGroupBy?: string;
  kanbanCardFields?: string[];
}

/** Generate YAML code for the current query. */
export function generateYamlCode(options: YamlGeneratorOptions): string {
  const {
    filters,
    matchMode,
    resultType,
    includeCompleted,
    viewType,
    kanbanGroupBy = "priority",
    kanbanCardFields = ["context", "due_date"],
  } = options;

  const validFilters = filters.filter((f) => f.key);

  let yaml = "```query\n";

  // Filters
  if (validFilters.length > 0) {
    yaml += "filters:\n";
    for (const filter of validFilters) {
      yaml += `  - key: ${filter.key}\n`;
      yaml += `    operator: ${filter.operator}\n`;
      if (filter.value && !VALUELESS_OPERATORS.includes(filter.operator)) {
        yaml += `    value: "${filter.value}"\n`;
      }
    }
  }

  // Match mode (only if multiple filters)
  if (validFilters.length > 1) {
    yaml += `match_mode: ${matchMode}\n`;
  }

  // Result type
  yaml += `result_type: ${resultType}\n`;

  // Include completed (only for tasks)
  if (resultType !== "Notes") {
    yaml += `include_completed: ${includeCompleted}\n`;
  }

  // View configuration
  yaml += "view:\n";
  yaml += `  view_type: ${viewType}\n`;

  if (viewType === "Kanban") {
    yaml += "  kanban:\n";
    yaml += `    group_by: ${kanbanGroupBy}\n`;
    yaml += "    card_fields:\n";
    for (const field of kanbanCardFields) {
      yaml += `      - ${field}\n`;
    }
    yaml += "    show_uncategorized: true\n";
  } else if (viewType === "Table") {
    if (resultType === "Notes") {
      yaml += "  columns:\n";
      yaml += "    - title\n";
      yaml += "    - path\n";
    } else {
      yaml += "  columns:\n";
      yaml += "    - description\n";
      yaml += "    - priority\n";
      yaml += "    - due_date\n";
      yaml += "    - note_title\n";
    }
  }

  yaml += "```";

  return yaml;
}

// ============================================================================
// Value Helpers
// ============================================================================

/** Split comma-separated values into individual items for list-type properties. */
export function splitListValues(values: string[]): string[] {
  const uniqueItems = new Set<string>();
  for (const value of values) {
    if (value.includes(", ")) {
      for (const item of value.split(", ")) {
        const trimmed = item.trim();
        if (trimmed) {
          uniqueItems.add(trimmed);
        }
      }
    } else {
      if (value.trim()) {
        uniqueItems.add(value.trim());
      }
    }
  }
  return [...uniqueItems].sort((a, b) => a.localeCompare(b));
}

/** Select option type for FuzzySelect. */
export interface SelectOption {
  value: string;
  label: string;
  group?: string;
  count?: number;
  suffix?: string;
}

/** Build options for the property key FuzzySelect. */
export function buildPropertyKeyOptions(propertyKeys: PropertyKeyInfo[]): SelectOption[] {
  const options: SelectOption[] = [];

  // Add special keys first
  for (const sk of SPECIAL_KEYS) {
    options.push({
      value: sk.key,
      label: sk.label,
      group: "Built-in",
    });
  }

  // Add property keys sorted alphabetically
  const sortedKeys = [...propertyKeys].sort((a, b) => a.key.localeCompare(b.key));
  for (const pk of sortedKeys) {
    const typeSuffix = pk.property_type ? ` (${pk.property_type})` : "";
    options.push({
      value: pk.key,
      label: pk.key,
      group: "Properties",
      count: pk.usage_count,
      suffix: typeSuffix,
    });
  }

  return options;
}
