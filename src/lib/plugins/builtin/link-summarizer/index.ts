/**
 * Link Summarizer Plugin
 *
 * Summarizes web pages using Puppeteer and a local LLM (LM Studio compatible).
 * Creates Obsidian-compatible markdown notes with frontmatter.
 */

import type { Plugin } from "../../types";
import type { LinkSummarizerSettings } from "./types";
import LinkSummarizerPanel from "./LinkSummarizerPanel.svelte";

export const linkSummarizerPlugin: Plugin<LinkSummarizerSettings> = {
  meta: {
    id: "link-summarizer",
    name: "Link Summarizer",
    description: "Summarize web pages using a local LLM and save as notes",
    version: "1.0.0",
    author: "NeuroFlow",
  },

  settingsSchema: [
    {
      title: "Output",
      description: "Path is relative to your vault root",
      fields: [
        {
          key: "outputDir",
          label: "Output Directory",
          description: "Directory to save summary notes (relative to vault)",
          type: "string",
          default: "Knowledge/Link Summaries",
          placeholder: "Knowledge/Link Summaries",
          required: true,
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
          placeholder: "openai/gpt-oss-20b",
        },
      ],
    },
    {
      title: "Defaults",
      fields: [
        {
          key: "defaultTags",
          label: "Default Tags",
          description: "Comma-separated tags to add to all summaries",
          type: "string",
          default: "article,summary,automated",
          placeholder: "article,summary,automated",
        },
      ],
    },
  ],

  defaultSettings: {
    outputDir: "Knowledge/Link Summaries",
    lmStudioEndpoint: "http://localhost:1234/v1/chat/completions",
    model: "",
    defaultTags: "article,summary,automated",
  },

  hooks: {
    sidebar: {
      panel: {
        id: "link-summarizer",
        label: "Link Summarizer",
        icon: "link",
        component: LinkSummarizerPanel,
      },
    },
  },

  async onEnable(settings) {
    console.log("Link Summarizer enabled with settings:", settings);
  },

  async onDisable() {
    console.log("Link Summarizer disabled");
  },

  async onSettingsChange(settings) {
    console.log("Link Summarizer settings changed:", settings);
  },
};

export default linkSummarizerPlugin;
