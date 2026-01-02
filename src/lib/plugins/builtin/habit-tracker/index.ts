/**
 * Habit Tracker Plugin
 *
 * Track habits with an embeddable table builder for interactive tracking.
 */

import type { Plugin } from "../../types";
import type { HabitTrackerSettings } from "./types";
import HabitTrackerPanel from "./HabitTrackerPanel.svelte";

export const habitTrackerPlugin: Plugin<HabitTrackerSettings> = {
  meta: {
    id: "habit-tracker",
    name: "Habit Tracker",
    description: "Track daily habits with embeddable interactive tables",
    version: "1.0.0",
    author: "NeuroFlow",
  },

  settingsSchema: [
    {
      title: "Default Settings",
      description: "Configure default behavior for habit tracker embeds",
      fields: [
        {
          key: "defaultView",
          label: "Default View",
          description: "Default view type when creating new embeds",
          type: "select",
          default: "table",
          options: [
            { value: "table", label: "Table" },
            { value: "calendar", label: "Calendar" },
            { value: "streak", label: "Streak" },
            { value: "list", label: "List" },
          ],
        },
        {
          key: "defaultDateRange",
          label: "Default Date Range",
          description: "Default date range for new embeds",
          type: "select",
          default: "last7_days",
          options: [
            { value: "last7_days", label: "Last 7 Days" },
            { value: "last30_days", label: "Last 30 Days" },
            { value: "this_week", label: "This Week" },
            { value: "this_month", label: "This Month" },
          ],
        },
        {
          key: "showInCalendar",
          label: "Show in Calendar",
          description: "Display habit completion indicators in calendar view",
          type: "boolean",
          default: true,
        },
        {
          key: "defaultColor",
          label: "Default Habit Color",
          description: "Default color for new habits (hex)",
          type: "string",
          default: "#6366f1",
          placeholder: "#6366f1",
        },
      ],
    },
  ],

  defaultSettings: {
    defaultView: "table",
    defaultDateRange: "last7_days",
    showInCalendar: true,
    defaultColor: "#6366f1",
  },

  hooks: {
    sidebar: {
      panel: {
        id: "habit-tracker",
        label: "Habit Tracker",
        icon: "check-circle",
        component: HabitTrackerPanel,
      },
    },
  },

  async onEnable(settings) {
    console.log("Habit Tracker enabled with settings:", settings);
  },

  async onDisable() {
    console.log("Habit Tracker disabled");
  },

  async onSettingsChange(settings) {
    console.log("Habit Tracker settings changed:", settings);
  },
};

export default habitTrackerPlugin;
