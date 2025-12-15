/**
 * LLM Summarizer Plugin
 *
 * Summarizes your day using a local LLM (LM Studio compatible).
 */

import type { Plugin } from "../../types";
import type { LLMSummarizerSettings } from "./types";
import SummarizerPanel from "./SummarizerPanel.svelte";

const DEFAULT_DAILY_PROMPT = `You are a helpful assistant that summarizes work days.
Given the schedule blocks and their associated notes, create a concise summary of what was accomplished.

Focus on:
- Key tasks completed
- Important meetings and their outcomes
- Progress made on projects
- Any blockers or issues encountered

Keep the summary professional and actionable.`;

const DEFAULT_CONFLUENCE_PROMPT = `You are a technical writer creating documentation for Confluence.
Given the raw notes and logs, transform them into clean, well-structured documentation.

Format the output as:
- Clear headings and sections
- Bullet points for key information
- Code blocks for any technical content
- Remove any personal notes or temporary content

Make it ready for publishing to Confluence.`;

const DEFAULT_STANDUP_PROMPT = `You are helping prepare a standup update.
Given yesterday's schedule and notes, create a brief standup update.

Format:
- What I did yesterday (2-3 bullet points)
- What I'm doing today (if applicable)
- Any blockers

Keep it concise and focused on key accomplishments.`;

export const llmFileSummarizerPlugin: Plugin<LLMSummarizerSettings> = {
  meta: {
    id: "llm-file-summarizer",
    name: "LLM Summarizer",
    description: "Summarize your day using a local LLM (LM Studio compatible)",
    version: "1.0.0",
    author: "NeuroFlow",
  },

  settingsSchema: [
    {
      title: "LLM Connection",
      description: "Configure the connection to your local LLM server",
      fields: [
        {
          key: "endpoint",
          label: "API Endpoint",
          description: "The OpenAI-compatible API endpoint (e.g., http://localhost:1234/v1)",
          type: "string",
          default: "http://localhost:1234/v1",
          placeholder: "http://localhost:1234/v1",
          required: true,
        },
        {
          key: "model",
          label: "Model",
          description: "The model to use (leave empty for default)",
          type: "string",
          default: "",
          placeholder: "local-model",
        },
        {
          key: "apiKey",
          label: "API Key",
          description: "API key if required (usually not needed for LM Studio)",
          type: "password",
          default: "",
          placeholder: "sk-...",
        },
      ],
    },
    {
      title: "Generation Settings",
      fields: [
        {
          key: "maxTokens",
          label: "Max Tokens",
          description: "Maximum tokens in the response",
          type: "number",
          default: 1000,
          min: 100,
          max: 4000,
        },
        {
          key: "temperature",
          label: "Temperature",
          description: "Controls randomness (0 = focused, 1 = creative)",
          type: "number",
          default: 0.7,
          min: 0,
          max: 1,
        },
        {
          key: "includeHistoryDays",
          label: "History Days",
          description: "Number of past days to include for context on recurring tasks",
          type: "number",
          default: 7,
          min: 0,
          max: 30,
        },
      ],
    },
    {
      title: "Prompts",
      description: "Customize the prompts for different summary types",
      fields: [
        {
          key: "dailySummaryPrompt",
          label: "Daily Summary Prompt",
          type: "textarea",
          default: DEFAULT_DAILY_PROMPT,
        },
        {
          key: "confluencePrompt",
          label: "Confluence Format Prompt",
          type: "textarea",
          default: DEFAULT_CONFLUENCE_PROMPT,
        },
        {
          key: "standupPrompt",
          label: "Standup Prompt",
          type: "textarea",
          default: DEFAULT_STANDUP_PROMPT,
        },
      ],
    },
  ],

  defaultSettings: {
    endpoint: "http://localhost:1234/v1",
    model: "",
    apiKey: "",
    maxTokens: 1000,
    temperature: 0.7,
    dailySummaryPrompt: DEFAULT_DAILY_PROMPT,
    confluencePrompt: DEFAULT_CONFLUENCE_PROMPT,
    standupPrompt: DEFAULT_STANDUP_PROMPT,
    includeHistoryDays: 7,
  },

  hooks: {
    calendar: {
      toolbarAction: {
        label: "Summarize Day",
        icon: "sparkles",
        onClick: async (selectedDate: string) => {
          // This will be handled by the SummarizerPanel component
          console.log("Summarize day:", selectedDate);
        },
      },
    },
    sidebar: {
      panel: {
        id: "llm-file-summarizer",
        label: "AI File Summary",
        icon: "sparkles",
        component: SummarizerPanel,
      },
    },
  },

  async onEnable(settings) {
    console.log("LLM Summarizer enabled with settings:", settings);
  },

  async onDisable() {
    console.log("LLM Summarizer disabled");
  },

  async onSettingsChange(settings) {
    console.log("LLM Summarizer settings changed:", settings);
  },
};

export default llmFileSummarizerPlugin;
