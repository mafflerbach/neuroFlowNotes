/**
 * Transcript Summarizer Plugin
 *
 * Summarizes YouTube transcripts using a local LLM (LM Studio compatible).
 * Creates Obsidian-compatible markdown notes with frontmatter.
 */

import type { Plugin } from "../../types";
import type { TranscriptSummarizerSettings } from "./types";
import TranscriptSummarizerPanel from "./TranscriptSummarizerPanel.svelte";

export const transcriptSummarizerPlugin: Plugin<TranscriptSummarizerSettings> = {
  meta: {
    id: "transcript-summarizer",
    name: "Transcript Summarizer",
    description: "Summarize YouTube transcripts using a local LLM",
    version: "1.0.0",
    author: "NeuroFlow",
  },

  settingsSchema: [
    {
      title: "Directories",
      description: "Paths are relative to your vault root",
      fields: [
        {
          key: "inputDir",
          label: "Input Directory",
          description: "Directory containing transcript .md files (relative to vault)",
          type: "string",
          default: "Knowledge/transcripts",
          placeholder: "Knowledge/transcripts",
          required: true,
        },
        {
          key: "outputDir",
          label: "Output Directory",
          description: "Directory to save summarized notes (relative to vault)",
          type: "string",
          default: "Knowledge/YT",
          placeholder: "Knowledge/YT",
          required: true,
        },
        {
          key: "assetTemplate",
          label: "Thumbnail Path Template",
          description: "Path template for thumbnails. Use {videoId} as placeholder.",
          type: "string",
          default: "",
          placeholder: "assets/{videoId}.jpg",
        },
      ],
    },
    {
      title: "LLM Connection",
      description: "Configure the connection to your local LLM server",
      fields: [
        {
          key: "lmStudioEndpoint",
          label: "LM Studio Endpoint",
          description: "The OpenAI-compatible chat completions endpoint",
          type: "string",
          default: "http://localhost:1234/v1/chat/completions",
          placeholder: "http://localhost:1234/v1/chat/completions",
          required: true,
        },
        {
          key: "model",
          label: "Model",
          description: "Model name (leave empty for default)",
          type: "string",
          default: "",
          placeholder: "gpt-oss-20b-uncensored",
        },
      ],
    },
  ],

  defaultSettings: {
    inputDir: "Knowledge/transcripts",
    outputDir: "Knowledge/YT",
    lmStudioEndpoint: "http://localhost:1234/v1/chat/completions",
    model: "",
    assetTemplate: "",
  },

  hooks: {
    sidebar: {
      panel: {
        id: "transcript-summarizer",
        label: "Transcript Summarizer",
        icon: "video",
        component: TranscriptSummarizerPanel,
      },
    },
  },

  async onEnable(settings) {
    console.log("Transcript Summarizer enabled with settings:", settings);
  },

  async onDisable() {
    console.log("Transcript Summarizer disabled");
  },

  async onSettingsChange(settings) {
    console.log("Transcript Summarizer settings changed:", settings);
  },
};

export default transcriptSummarizerPlugin;
