/**
 * Query API - property-based query builder operations
 */

import { invoke } from "@tauri-apps/api/core";
import type {
  PropertyKeyInfo,
  QueryRequest,
  QueryResponse,
  QueryEmbedResponse,
} from "../../types";

/**
 * Get all property keys used in the vault (for query builder dropdown).
 */
export async function getPropertyKeys(): Promise<PropertyKeyInfo[]> {
  return invoke<PropertyKeyInfo[]>("get_property_keys");
}

/**
 * Get all distinct values for a property key (for autocomplete).
 */
export async function getPropertyValues(key: string): Promise<string[]> {
  return invoke<string[]>("get_property_values", { key });
}

/**
 * Get all distinct individual values for a list-type property.
 * Splits comma-separated values and returns unique items.
 * Use this for ContainsAny/ContainsAll operators on list properties.
 */
export async function getListPropertyValues(key: string): Promise<string[]> {
  return invoke<string[]>("get_list_property_values", { key });
}

/**
 * Run a query with property filters.
 */
export async function runQuery(request: QueryRequest): Promise<QueryResponse> {
  return invoke<QueryResponse>("run_query", { request });
}

/**
 * Execute a query embed from YAML content.
 * This parses the YAML and executes the query, returning both the parsed config and results.
 */
export async function executeQueryEmbed(yamlContent: string): Promise<QueryEmbedResponse> {
  return invoke<QueryEmbedResponse>("execute_query_embed", { yamlContent });
}
