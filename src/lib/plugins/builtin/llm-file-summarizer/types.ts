/**
 * LLM Summarizer Plugin Types
 */

export interface LLMSummarizerSettings {
  endpoint: string;
  model: string;
  apiKey: string;
  maxTokens: number;
  temperature: number;
  dailySummaryPrompt: string;
  confluencePrompt: string;
  standupPrompt: string;
  includeHistoryDays: number;
}

export interface SummaryRequest {
  date: string;
  includeHistory: boolean;
  outputFormat: "daily" | "confluence" | "standup";
}

export interface BlockWithContent {
  blockId: number;
  date: string;
  startTime: string;
  endTime: string;
  label: string | null;
  noteId: number | null;
  notePath: string | null;
  noteTitle: string | null;
  noteContent: string | null;
}

export interface SummaryResult {
  date: string;
  blocks: BlockWithContent[];
  summary: string;
  tokensUsed?: number;
}
