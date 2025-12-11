/**
 * EditorCache - Generic cache utility for CodeMirror extensions.
 * Provides TTL-based caching to avoid repeated API calls on every keystroke.
 */

interface CacheEntry<T> {
  data: T;
  timestamp: number;
}

/**
 * A generic cache with time-to-live expiration.
 */
export class EditorCache<T> {
  private cache = new Map<string, CacheEntry<T>>();

  /**
   * Create a new cache with the specified TTL.
   * @param ttlMs Time-to-live in milliseconds
   */
  constructor(private ttlMs: number) {}

  /**
   * Get a cached value if it exists and hasn't expired.
   */
  get(key: string): T | null {
    const entry = this.cache.get(key);
    if (!entry) return null;

    if (Date.now() - entry.timestamp > this.ttlMs) {
      this.cache.delete(key);
      return null;
    }

    return entry.data;
  }

  /**
   * Set a value in the cache.
   */
  set(key: string, data: T): void {
    this.cache.set(key, { data, timestamp: Date.now() });
  }

  /**
   * Check if a key exists and hasn't expired.
   */
  has(key: string): boolean {
    return this.get(key) !== null;
  }

  /**
   * Delete a specific key from the cache.
   */
  delete(key: string): boolean {
    return this.cache.delete(key);
  }

  /**
   * Clear all cached entries.
   */
  clear(): void {
    this.cache.clear();
  }

  /**
   * Get the number of cached entries.
   */
  get size(): number {
    return this.cache.size;
  }

  /**
   * Get or fetch a value - returns cached value if valid, otherwise calls fetcher.
   */
  async getOrFetch(key: string, fetcher: () => Promise<T>): Promise<T> {
    const cached = this.get(key);
    if (cached !== null) {
      return cached;
    }

    const data = await fetcher();
    this.set(key, data);
    return data;
  }
}
