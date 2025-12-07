
# NeuroFlow Notes – Workspace Layout & View States

> **Implementation Status:** ✅ UI Shell Complete (v0.2.0)
>
> All layout components implemented. Data wiring in progress.

Dieses Dokument beschreibt das grobe UI-Layout und die Zustände des Workspaces.
Der Fokus liegt auf:

- Kalender-zentriertem Einstieg
- Umschalten in einen Dokument-/Finder-Modus
- Klar getrennten Layouts für Kalender- und Dokumentarbeit
- Einfache, konsistente Interaktionen über die Icon-Leiste

---

## 1. Icon-Leiste (globale Topbar)

Oben, über allen Views:

```text
[ ≡ ] [ M ] [ W ] [ •Today ] | [ +Note ] | [ ⚙ ]
```

**Implemented in:** `src/lib/components/Topbar.svelte`

* **[ ≡ ]** – Toggle Folderview (ein-/ausblenden)
* **[ M ]** – Monthly View des Kalenders
* **[ W ]** – Weekly View des Kalenders
* **[ •Today ]** – Daily View des Kalenders für den aktuellen Tag
* **[ +Note ]** – neue Note erstellen (z.B. `new-{timestamp}.md` im Root) und im Editor öffnen
* **[ ⚙ ]** – Settings als Modal (Obsidian-Style), nicht als eigener Tab

**Regel:**
Ein Klick auf einen der Kalender-Buttons (`M`, `W`, `•Today`) bringt den User immer in die entsprechende Kalender-Ansicht (Kalender-View 1 oder 2, siehe unten).

---

## 2. Kalenderview ohne Dokument (State A)

**Standard-Einstieg**, wenn kein Dokument offen ist.

**State:** `workspaceStore.state === "calendar-only"`

### Layout

```text
Topbar:
[ M ] [ W ] [ •Today ] | [ +Note ] | [ ⚙ ]

┌───────────────────────────────────────────────────────────────────────┐
│ FOLDERVIEW (ca. 15%) │        KALENDER (Restbreite)                  │
│                       │----------------------------------------------│
│ [vault/]              │  Weekly View (Outlook-Style)                 │
│   daily/              │                                              │
│   projects/           │  | Mon | Tue | Wed | Thu | Fri | Sat | Sun | │
│   areas/              │  |-----|-----|-----|-----|-----|-----|-----| │
│                       │  |     |  ■  |     |  █  |     |  ·  |  ·  | │
│ (togglebar,           │  |     |docs|     |appt|     |docs|docs|   | │
│  ein-/ausblendbar)    │                                              │
└───────────────────────┴──────────────────────────────────────────────┘
```

### Kalender-Ansichten

* **Monthly View** → `CalendarMonthly.svelte`

  * Nur Dots pro Tag (`·`) für vorhandene Notes.
  * Liste aller Docs des Monats (unter dem Kalender-Grid).

* **Weekly View** → `CalendarWeekly.svelte`

  * Appointments als Time Blocks (`█`) – colored schedule blocks.
  * Notes mit Terminbezug als markierte Dots/Icons (`■`).
  * Notes ohne direkten Terminbezug als einfache Dots (`·`).
  * Hour slots (06:00–22:00) mit 48px Höhe.

* **Daily View** → `CalendarDaily.svelte`

  * Vertikale Timeline (06:00–22:00).
  * Time Blocks für Termine (full colored slots).
  * Current time indicator (rote Linie).
  * Alle Docs dieses Tages in einer Liste → `DocList.svelte`

---

## 3. Kalenderview mit einem offenen Dokument (State B)

Ein Dokument wurde aus dem Kalender (oder Folder) geöffnet, der Kalender bleibt sichtbar.

**State:** `workspaceStore.state === "calendar-with-doc"`

### Layout

```text
Topbar:
[ M ] [ W ] [ •Today ] | [ +Note ] | [ ⚙ ]

┌───────────────────────────────────────────────────────────────────────────────┐
│ FOLDERVIEW (15%)  │   KALENDER (ca. 20–25%)   │          DOC 1 CONTENT      │
│                    │--------------------------│-----------------------------│
│ [vault/]           │  Tages- oder Wochenansicht│  # Title of doc_1          │
│   daily/           │  mit Time Blocks & Dots   │  ...                       │
│   projects/        │                          │  Markdown-Editor            │
│   areas/           │                          │                             │
│                    │                          │-----------------------------│
│                    │                          │  DOCS FÜR DIESEN TAG (⊟)    │
│                    │                          │  - doc_1 (current)         │
│                    │                          │  - doc_2                   │
│                    │                          │  - doc_3                   │
│                    │                          │-----------------------------│
│                    │                          │  PROPERTIES (togglebar)     │
│                    │                          │  key: value                │
│                    │                          │  status: in-progress       │
└────────────────────┴--------------------------┴-----------------------------┘
```

### Verhalten

* **Folderview**

  * Klick auf Datei → Doc wird im rechten Editor geöffnet, Kalender bleibt sichtbar.
  * Die aktuell geöffnete Datei wird im Baum hervorgehoben.
* **Kalender**

  * Klick auf Time Block oder Doc-Dot:

    * öffnet die verlinkte Note im Editor rechts.
* **Liste der Docs für den Tag**

  * Togglebar (ein-/ausklappbar).
  * Klick auf einen Eintrag öffnet entsprechende Note im Editor.
* **Properties** → `PropertiesPanel.svelte`

  * Unten im Doc-Bereich als Panel.
  * Per Toggle fährt das Properties-Panel hoch (z.B. auf 50% Höhe).
  * Key/Value-Stil, z.B. `type`, `status`, `tags`, `journal_date`, etc.
  * **Wichtig:** Properties werden nur in SQLite gespeichert, nicht als YAML-Frontmatter im Markdown (zu noisy).

---

## 4. Doc View / Finder-Modus (State C)

Sobald im Editor auf einen `[[Link]]` geklickt wird, verschwindet der Kalender.
Der Fokus wechselt in den **Dokumenten-Explorationsmodus** mit Finder-artiger Navigation.

**State:** `workspaceStore.state === "doc-finder"`

**Components:**
- `Breadcrumb.svelte` – Navigation trail mit collapse indicator
- `DocumentColumns.svelte` – Multi-column editor (sliding window, max 3)

### Top-Layout

```text
Topbar:
[ M ] [ W ] [ •Today ] | [ +Note ] | [ ⚙ ]

────────────────────────────────────────────────────────────────────────
BREADCRUMB:
doc_1.md  ›  doc_2.md  ›  doc_3.md  ›  ...

────────────────────────────────────────────────────────────────────────
```

Die Breadcrumb zeigt den **vollen Denkpfad**: jede Note, die über einen Link geöffnet wurde, wird angehängt.

### Hauptlayout (2 oder 3 Docs sichtbar, Sliding-Window)

Beispiel mit 2 Docs:

```text
┌────────────────────────────────────────────────────────────────────────┐
│ FOLDERVIEW (15%)  │           DOC 1 CONTENT         │   DOC 2 CONTENT │
│                   │----------------------------------│----------------│
│ [vault/]          │  # doc_1                        │  # doc_2        │
│   daily/          │  ...                            │  ...            │
│   projects/       │  [[link to doc_2]]              │  [[link doc_3]] │
│   areas/          │                                  │                │
│                   │----------------------------------│----------------│
│                   │  PROPERTIES DOC 1               │ PROPERTIES DOC 2│
│                   │  key: value                     │ key: value      │
└───────────────────┴----------------------------------┴----------------┘
```

Mit 3 Docs (maximal):

```text
FOLDERVIEW |   DOC A   |   DOC B   |   DOC C
```

Alle Doc-Spalten teilen den verfügbaren Platz (nach Folderview) **gleichmäßig**.

### Sliding-Window-Regel

* Interner Zustand (Beispiel):

  ```text
  breadcrumb = [note1, note2, note3]
  visibleDocs = [note1, note2, note3]
  ```
* Klick auf Link zu `note4` in `note3`:

  * `breadcrumb = [note1, note2, note3, note4]`
  * `visibleDocs = [note2, note3, note4]`
* Es sind immer maximal **3 Doc-Spalten** sichtbar:

  * `visibleDocs = last 3 entries of breadcrumb`

Klick auf einen Eintrag in der Breadcrumb:

* setzt `activeNote = gewählt`
* schneidet den Breadcrumb rechts davon ab
* berechnet `visibleDocs = last 3 of breadcrumb` neu

---

## 5. Icon-Verhalten (Transitions zwischen States)

### [ M ], [ W ], [ •Today ]

* Egal ob in State A (nur Kalender), State B (Kalender + Doc) oder State C (Doc/Finder):

  * Klick bringt zurück in den **Kalender-Workspace**.
  * Layout: Folderview + Kalender (Weekly/Monthly/Daily).
  * Optional: wenn eine Note aktiv war, kann diese in State B weiterhin im Editor angezeigt werden.

### [ +Note ] (note icon)

* Öffnet (oder zeigt) die Folderview.
* Erzeugt eine neue Datei, z.B.:

  * `new.md` im Root der Vault.
* Öffnet `new.md` im Editor:

  * Im Kalender-Modus: rechts neben Kalender.
  * Im Doc/Finder-Modus: als neue aktive Note (in einer der Doc-Spalten).

### [ ⚙ ] (settings icon)

* Öffnet ein zentrales **Settings-Modal** (kein Tab):

  * App Settings (global, `app.db`)
  * Vault Settings (pro Vault, `content.db` / config)
  * Plugins
  * Appearance etc.

---

## 6. Zustandsüberblick (A → B → C)

* **State A – Kalender, kein Doc**

  * Folderview + Kalender
  * Noch kein Doc im Editor aktiv.

* **State B – Kalender + Doc**

  * Folderview + Kalender + Editor für 1 Note.
  * Docs für den Tag + Properties als Toggles im Editor-Bereich.

* **State C – Doc/Finder-Modus**

  * Kalender weg.
  * Folderview links, 2–3 Doc-Spalten rechts.
  * Breadcrumb zeigt kompletten Denkpfad.
  * Properties pro Doc-Spalte unten.

Übergänge:

* A → B:

  * Klick auf Time Block / Doc-Dot im Kalender
  * Klick auf Datei in der Folderview

* B → C:

  * Klick auf `[[Link]]` im Editor

* C → A/B:

  * Klick auf einen der Kalender-Buttons `[ M ] [ W ] [ •Today ]`

---

## 7. Settings

**Component:** `SettingsModal.svelte`

### Multi-Column Editing Mode

User-configurable setting (`workspaceStore.multiColumnEditable`):

* **Enabled (default):** Alle sichtbaren Doc-Spalten sind editierbar.
* **Disabled:** Nur die aktive Spalte ist editierbar, andere sind read-only.

---

## 8. Implementation Summary

| Component | File | Status |
|-----------|------|--------|
| Topbar | `Topbar.svelte` | ✅ |
| Workspace Store | `workspace.svelte.ts` | ✅ |
| Calendar Monthly | `CalendarMonthly.svelte` | ✅ |
| Calendar Weekly | `CalendarWeekly.svelte` | ✅ |
| Calendar Daily | `CalendarDaily.svelte` | ✅ |
| DocList | `DocList.svelte` | ✅ |
| Properties Panel | `PropertiesPanel.svelte` | ✅ |
| Breadcrumb | `Breadcrumb.svelte` | ✅ |
| Document Columns | `DocumentColumns.svelte` | ✅ |
| Settings Modal | `SettingsModal.svelte` | ✅ |
| App Layout | `App.svelte` | ✅ |

### Pending: Data Wiring

- [ ] Properties API (backend)
- [ ] Schedule Blocks API (backend)
- [ ] Notes by Date query (backend)
- [ ] Wire calendar components to real data

