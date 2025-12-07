/**
 * API service layer - wraps Tauri commands for the frontend.
 *
 * This file re-exports from domain-specific modules in ./api/
 * for backward compatibility. New code should import directly
 * from the specific modules when possible.
 */

export * from "./api/index";
