/**
 * Vault and folder types
 */

export interface VaultInfo {
  path: string;
  name: string;
  note_count: number;
}

export interface FolderNode {
  name: string;
  path: string;
  is_dir: boolean;
  children: FolderNode[];
}
