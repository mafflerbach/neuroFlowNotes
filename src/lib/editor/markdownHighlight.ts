/**
 * Custom markdown syntax highlighting for CodeMirror
 * Uses CSS variables for theme-aware colors
 */

import { HighlightStyle, syntaxHighlighting } from "@codemirror/language";
import { tags } from "@lezer/highlight";
import {
  EditorView,
  ViewPlugin,
  Decoration,
} from "@codemirror/view";
import type { ViewUpdate, DecorationSet } from "@codemirror/view";
import { RangeSetBuilder } from "@codemirror/state";
import { syntaxTree } from "@codemirror/language";

/**
 * Custom highlight style for markdown that uses CSS variables
 */
const markdownHighlightStyle = HighlightStyle.define([
  // Headings
  {
    tag: tags.heading1,
    fontSize: "1.75em",
    fontWeight: "bold",
    color: "var(--md-heading-1)",
    lineHeight: "1.3",
  },
  {
    tag: tags.heading2,
    fontSize: "1.5em",
    fontWeight: "bold",
    color: "var(--md-heading-2)",
    lineHeight: "1.35",
  },
  {
    tag: tags.heading3,
    fontSize: "1.3em",
    fontWeight: "600",
    color: "var(--md-heading-3)",
    lineHeight: "1.4",
  },
  {
    tag: tags.heading4,
    fontSize: "1.15em",
    fontWeight: "600",
    color: "var(--md-heading-4)",
  },
  {
    tag: tags.heading5,
    fontSize: "1.05em",
    fontWeight: "600",
    color: "var(--md-heading-5)",
  },
  {
    tag: tags.heading6,
    fontSize: "1em",
    fontWeight: "600",
    color: "var(--md-heading-6)",
  },

  // Emphasis
  {
    tag: tags.emphasis,
    fontStyle: "italic",
    color: "var(--md-italic)",
  },
  {
    tag: tags.strong,
    fontWeight: "bold",
    color: "var(--md-bold)",
  },
  {
    tag: tags.strikethrough,
    textDecoration: "line-through",
    color: "var(--md-strikethrough)",
  },

  // Links
  {
    tag: tags.link,
    color: "var(--md-link)",
    textDecoration: "underline",
  },
  {
    tag: tags.url,
    color: "var(--md-link-url)",
  },

  // Code - inline and blocks
  {
    tag: tags.monospace,
    fontFamily: "var(--font-family-mono)",
    color: "var(--md-code-text)",
    backgroundColor: "var(--md-code-bg)",
  },

  // Quotes
  {
    tag: tags.quote,
    color: "var(--md-blockquote)",
    fontStyle: "italic",
  },

  // Lists
  {
    tag: tags.list,
    color: "var(--md-list-marker)",
  },

  // Meta/processing instructions (like --- for frontmatter, ``` for code blocks)
  {
    tag: tags.meta,
    color: "var(--md-syntax-marker)",
  },

  // Content separator (---)
  {
    tag: tags.contentSeparator,
    color: "var(--border-default)",
  },

  // Formatting characters (*, _, #, etc.)
  {
    tag: tags.processingInstruction,
    color: "var(--md-syntax-marker)",
  },

  // Code block specific - keywords, strings, etc.
  {
    tag: tags.keyword,
    color: "var(--md-code-keyword)",
  },
  // Strings - using theme-aware colors
  {
    tag: tags.string,
    color: "var(--md-code-string)",
  },
  {
    tag: tags.special(tags.string),
    color: "var(--md-code-string)",
  },
  {
    tag: tags.character,
    color: "var(--md-code-string)",
  },
  {
    tag: tags.docString,
    color: "var(--md-code-string)",
  },
  {
    tag: tags.regexp,
    color: "var(--md-code-string)",
  },
  {
    tag: tags.escape,
    color: "var(--md-code-string)",
  },
  {
    tag: tags.number,
    color: "var(--md-code-number)",
  },
  {
    tag: tags.integer,
    color: "var(--md-code-number)",
  },
  {
    tag: tags.float,
    color: "var(--md-code-number)",
  },
  {
    tag: tags.comment,
    color: "var(--md-code-comment)",
    fontStyle: "italic",
  },
  {
    tag: tags.lineComment,
    color: "var(--md-code-comment)",
    fontStyle: "italic",
  },
  {
    tag: tags.blockComment,
    color: "var(--md-code-comment)",
    fontStyle: "italic",
  },
  {
    tag: tags.function(tags.variableName),
    color: "var(--md-code-function)",
  },
  {
    tag: tags.definition(tags.variableName),
    color: "var(--md-code-variable)",
  },
  {
    tag: tags.variableName,
    color: "var(--md-code-variable)",
  },
  {
    tag: tags.operator,
    color: "var(--md-code-operator)",
  },
  {
    tag: tags.punctuation,
    color: "var(--md-code-punctuation)",
  },
  {
    tag: tags.bracket,
    color: "var(--md-code-bracket)",
  },
  {
    tag: tags.paren,
    color: "var(--md-code-bracket)",
  },
  {
    tag: tags.squareBracket,
    color: "var(--md-code-bracket)",
  },
  {
    tag: tags.brace,
    color: "var(--md-code-bracket)",
  },
  {
    tag: tags.className,
    color: "var(--md-code-class)",
  },
  {
    tag: tags.typeName,
    color: "var(--md-code-class)",
  },
  {
    tag: tags.propertyName,
    color: "var(--md-code-property)",
  },
  {
    tag: tags.bool,
    color: "var(--md-code-boolean)",
  },
  {
    tag: tags.null,
    color: "var(--md-code-null)",
  },
  {
    tag: tags.atom,
    color: "var(--md-code-boolean)",
  },
  {
    tag: tags.self,
    color: "var(--md-code-keyword)",
  },
  // Catch-all for any remaining code elements
  {
    tag: tags.name,
    color: "var(--md-code-variable)",
  },
]);

/**
 * Decoration for lines inside fenced code blocks
 */
const codeBlockLineDecoration = Decoration.line({
  class: "cm-codeblock-line",
});

/**
 * ViewPlugin that adds background decoration to code block lines
 */
const codeBlockPlugin = ViewPlugin.fromClass(
  class {
    decorations: DecorationSet;

    constructor(view: EditorView) {
      this.decorations = this.buildDecorations(view);
    }

    update(update: ViewUpdate) {
      if (update.docChanged || update.viewportChanged) {
        this.decorations = this.buildDecorations(update.view);
      }
    }

    buildDecorations(view: EditorView): DecorationSet {
      const builder = new RangeSetBuilder<Decoration>();
      const tree = syntaxTree(view.state);

      // Find all FencedCode nodes
      tree.iterate({
        enter: (node) => {
          if (node.name === "FencedCode") {
            // Get all lines within this code block
            const doc = view.state.doc;
            let pos = node.from;

            while (pos <= node.to) {
              const line = doc.lineAt(pos);
              builder.add(line.from, line.from, codeBlockLineDecoration);

              if (line.to >= node.to) break;
              pos = line.to + 1;
            }
          }
        },
      });

      return builder.finish();
    }
  },
  {
    decorations: (v) => v.decorations,
  }
);

/**
 * Extension that provides markdown syntax highlighting
 */
export function markdownHighlight() {
  return [
    // Apply inline styles with CSS variables for theme-aware highlighting
    syntaxHighlighting(markdownHighlightStyle),
    // Add background to code block lines
    codeBlockPlugin,
  ];
}
