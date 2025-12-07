/**
 * Properties API - note property operations
 */

import { invoke } from "@tauri-apps/api/core";
import type { PropertyDto, SetPropertyRequest } from "../../types";

export async function getProperties(noteId: number): Promise<PropertyDto[]> {
  return invoke<PropertyDto[]>("get_properties", { noteId });
}

export async function setProperty(request: SetPropertyRequest): Promise<number> {
  return invoke<number>("set_property", { request });
}

export async function deleteProperty(noteId: number, key: string): Promise<void> {
  return invoke("delete_property", { noteId, key });
}
