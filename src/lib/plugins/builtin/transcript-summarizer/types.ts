/**
 * Transcript Summarizer Plugin Types
 */

export interface TranscriptSummarizerSettings {
  inputDir: string;
  outputDir: string;
  lmStudioEndpoint: string;
  model: string;
  assetTemplate: string;
}

export interface SummarizerResult {
  success: boolean;
  processed: number;
  failed: number;
  output_lines: string[];
}

export interface SummarizerProgress {
  type: string;
  file?: string;
  index?: number;
  total?: number;
  stage?: string;
  error?: string;
  source?: string;
  output?: string;
  video_id?: string;
  thumbnail?: string;
}
