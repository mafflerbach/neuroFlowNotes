/**
 * Link Summarizer Plugin Types
 */

export interface LinkSummarizerSettings {
  outputDir: string;
  lmStudioEndpoint: string;
  model: string;
  defaultTags: string;
}

export interface SummarizerResult {
  success: boolean;
  processed: number;
  failed: number;
  output_lines: string[];
}

export interface SummarizerProgress {
  type: string;
  url?: string;
  index?: number;
  stage?: string;
  error?: string;
  filepath?: string;
  title?: string;
  filename?: string;
}
