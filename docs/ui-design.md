# UI Design

> **Document state:** design contract. The token values are implemented in
> `crates/kaffein-ui/src/theme.rs` (2026-10-26) and this table mirrors them. Layout and component
> requirements remain design intent; no visual or contrast check has been re-run against the
> running build.

## Design intent

Build a desktop IDE for Java development with egui. IntelliJ IDEA is the visual and interaction
reference: an information-dense, keyboard-first workspace where the code is the loudest thing on
screen and everything else recedes.

Do not copy IntelliJ's proprietary assets, icons, fonts, or source. Recreate general interaction
patterns and visual language independently.

Five constraints drive most decisions:

1. **The editor is the product.** Chrome earns its space or gives it back.
2. **Density with air.** Rows are compact (22 px), but groups are separated by spacing, not boxes.
3. **Keyboard first, mouse never blocked.** Every action is reachable without a pointer, and every
   shortcut is a registered command.
4. **No fake affordances.** A control that cannot act is disabled with a reason, not drawn as if it
   works.
5. **State is always visible.** Focus, selection, modified documents, running jobs, and diagnostics
   are legible at a glance without opening a dialog.

## Layout

```
┌──────────────────────────────────────────────────────────────────────────────┐
│ Menu bar          File  Edit  View  Navigate  Code  Analyze  Refactor  …     │ 28
├──────────────────────────────────────────────────────────────────────────────┤
│ Toolbar   [Run] [Debug] [Stop]        breadcrumb: src › main › java › App     │ 34
├───────────────────┬──────────────────────────────────────────────────────────┤
│ Project           │  App.java ×   Service.java •                              │ 30  tabs
│ ▾ src             ├──────────────────────────────────────────────────────────┤
│   ▾ main          │  1  package com.example;                                  │
│     ▾ java        │  2                                                        │
│       App.java    │  3  public final class App {                              │ flexible
│     ▸ resources   │  4      public static void main(String[] args) {          │
│ ▸ test            │  5          System.out.println("hello");                  │
│                   │  6      }                                                 │
│                   │  7  }                                                     │
├───────────────────┼──────────────────────────────────────────────────────────┤
│                   │  Problems  │  Build  │  Terminal  │  Version Control      │ 26  tab strip
│                   │  (content of the selected bottom tool window)             │ 160+ resizable
├───────────────────┴──────────────────────────────────────────────────────────┤
│ main ↑2   ⚠ 0  ✖ 1   Java 21   Gradle   UTF-8   LF   English   indexing…  │ 24
└──────────────────────────────────────────────────────────────────────────────┘
```

Regions and rules:

| Region | Default size | Rules |
|---|---|---|
| Menu bar | 28 px | Complete action surface; never the only route to an action |
| Toolbar | 34 px | Primary actions only; overflow into a chevron, never clip |
| Left tool window (Project) | 280 px, min 180, max 45% of width | Resizable, collapsible with a keyboard shortcut; remembers width |
| Editor tabs | 30 px | Dirty marker, close affordance, overflow list; middle-click closes |
| Editor surface | flexible | Never below 1 visible line plus gutter; horizontal scroll, no wrapping by default |
| Bottom tool window | 160 px, min 120 | Shares the editor's horizontal space; tabs on top, not a dropdown |
| Bottom tool window tabs | 26 px | Tabs, badge counts, close affordance; keep the tab order stable across sessions |
| Status bar | 24 px | Read-only summaries that are clickable when they have a destination |

Panels resize with a visible splitter hit area (6 px) drawn in `border_strong`, and a minimum size.
Splitters must not eat clicks meant for the scrollbar.

## Design tokens

Keep shared values centralized in one theme module. Widgets consume semantic tokens; they never
inline a colour or size. The table is the **contract for names**; the code is the source of truth
for values.

| Token | Role | Dark baseline | Light baseline |
|---|---|---|---|
| `surface_editor` | Editor background | `#1E1F22` | `#FFFFFF` |
| `surface_panel` | Tool window and panel background | `#2B2D30` | `#F4F5F7` |
| `surface_raised` | Popups, menus, hovered rows | `#35373B` | `#FFFFFF` |
| `surface_sunken` | Status bar, inset wells | `#232528` | `#ECEEF1` |
| `border` | Decorative panel outlines | `#43454A` | `#D3D5DB` |
| `border_subtle` | List separators, indent guides | `#313337` | `#E4E6EA` |
| `border_strong` | Essential boundaries: splitter handles, input outlines | `#767A82` | `#7A7F88` |
| `text_primary` | Body and code text | `#DFE1E5` | `#1F2328` |
| `text_muted` | Secondary labels, hints | `#9DA0A8` | `#626A78` |
| `text_disabled` | Disabled labels | `#6A6D73` | `#A3A7AE` |
| `accent` | Primary action, links, focus | `#8AB4FF` | `#2258D8` |
| `accent_muted` | Accent fills behind text | `#1E3A6E` | `#DBEAFE` |
| `selection_row` | Selected project/tree/list row background | `#2E436E` | `#D6E4FF` |
| `selection_editor` | Editor text selection background | `#2E436E` | `#D6E4FF` |
| `selection_text` | Foreground on selected rows and selections | `#E8EAED` | `#1F2328` |
| `selection_inactive` | Selection in an unfocused surface | `#3A3D42` | `#E8EAEE` |
| `error` | Errors, failing build | `#FF7A85` | `#C4314B` |
| `warning` | Warnings | `#E5A13A` | `#94590A` |
| `success` | Passing build, saved state | `#62B543` | `#2A7430` |
| `info` | Informational diagnostics | `#6BA6FF` | `#1D4ED8` |
| `diff_added` | Inserted line background | `#1F3A2B` | `#E6FFEC` |
| `diff_removed` | Removed line background | `#45262B` | `#FFEBE9` |

Additional token groups:

| Category | Tokens | Baseline |
|---|---|---|
| Typography | see [Typography](#typography) | 13 px UI, 13 px code |
| Spacing | `space_1`…`space_7` | 2, 4, 6, 8, 12, 16, 24 px |
| Radii | `radius_sm`, `radius_md`, `radius_lg` | 3, 4, 6 px |
| Borders | `border_width`, `focus_width` | 1 px, 2 px |
| Row heights | `row_compact`, `row_standard` | 22 px, 28 px |
| Motion | `motion_fast`, `motion_base` | 120 ms, 180 ms |
| Elevation | popup shadow blur / offset | 12 px / 4 px |

Rules:

- Use semantic names (`text_muted`), never colour names (`grey_500`) and never raw values in widgets.
- One token, one role family. A family is a set of surfaces that intentionally share a value —
  `surface_raised` covers popups, menus, and hovered rows. The Role cell must list every member. If
  one member needs a different value in either theme, split the token instead of special-casing the
  widget.
- A new token requires a reason in the change description; a new raw value in a widget does not.
- Syntax highlighting tokens are separate from chrome tokens so editor themes can change
  independently.

The theme module is the source of truth: `crates/kaffein-ui/src/theme.rs`. The dark and light
columns above are mirrored from it on 2026-10-26; the values are `Implemented`, not `Verified`.
The `view.toggle_theme` command switches between the two sets at runtime.

Contrast of these values was computed on 2026-10-26 with the WCAG 2.1 relative-luminance formula.
Every foreground token (`text_primary`, `text_muted`, `selection_text`, `accent`, `error`,
`warning`, `success`, `info`) reaches at least 4.5:1 against `surface_editor`, `surface_panel`,
`surface_raised`, and `surface_sunken` in both themes. `text_disabled` is exempt as an inactive
control. `border` and `border_subtle` are decorative separators; `border_strong` is the token for
essential boundaries (splitter handles, input outlines) and reaches at least 3:1. Re-measure against
a captured frame from the shipped theme module before marking any row `Verified`.

Baseline revised on 2026-10-26 to meet that target: `accent`, `error`, `success`, `warning`, and the
light `text_muted` changed values; `selection_text` became a foreground token, `selection_editor`
was added for the selection background, and `border_strong` was added so splitter handles are not
drawn with a decorative border. The previous values were never measured against a build.

## Typography

| Role | Size | Line height | Notes |
|---|---|---|---|
| Code | 13 px | 18 px | Monospace with a programming ligature set, optional |
| UI label | 13 px | 18 px | Proportional sans |
| UI small / status bar | 12 px | 16 px | Secondary information only, never primary actions |
| Tab title | 13 px | 18 px | Ellipsis in the middle for long paths |
| Section header | 13 px | 18 px | Uppercase with letter-spacing ≥ 0.4 px |
| Dialog title | 15 px | 20 px | Sentence case |
| Welcome / empty state | 20 px | 28 px | One per screen, not decorative |
| Code comment / inlay hint | 12 px | 18 px | Muted colour, must still pass contrast |

Rules:

- Respect the OS/egui scale factor; never hard-code pixel sizes that ignore `pixels_per_point`.
- Provide a configurable UI font size (and a separate editor font size); test at 100%, 125%, 150%.
- Monospace is for code, paths, and identifiers — not for prose or buttons.
- Numeric columns (line numbers, counts) align to the right or use tabular figures.

## Localisation

Copy is data. Every menu title, toolbar label, tooltip, status message, prompt, and error string is
looked up through the injected `Localizer`; widgets do not embed English text. The mechanism, key
conventions, catalogue metadata, and file locations are documented in
[`architecture.md`](architecture.md) › Localisation; this section fixes the visible behaviour.

The **Language** selector is a submenu of **View**. It lists every loaded locale by its own name
(`English`, `Português (Brasil)`), marks the active one, and dispatches `app.set_locale` with the
locale code. The status bar shows the active locale name in its right-hand group. Selecting a language
re-renders menus, panels, prompts, and the status message already on screen on the next frame; no
restart is required.

Rules:

- Add a key constant and its entry in every bundled catalog in the same change; a widget may only
  reference the key.
- Keep a sentence in one catalog entry with `{name}` placeholders. Do not concatenate translated
  fragments, because word order and grammar differ between languages.
- File names, project names, paths, counts, and error details are values substituted into a
  localised template, not translated text.
- The bundled catalogs are `en` and `pt-BR`. A further language is a `locales/<code>.json` file with
  a `meta` block, not a widget branch or a hard-coded selector list.
- Locale names are written in the language itself and are never translated into the active UI
  language.

## Components

| Component | Anatomy | States to define | Rules |
|---|---|---|---|
| Menu bar | Top-level items, popup panels, shortcut hints, separators | open, hover, disabled | Every item maps to a command; shortcuts render from the registry |
| Language selector | **View** submenu, locale names, active marker | open, hover, selected | Generated from the loaded catalogs; selecting a name dispatches `app.set_locale` |
| Toolbar | Icon buttons, overflow chevron | default, hover, pressed, disabled, running | Icons only with tooltips that include the shortcut |
| Project tree | Rows, chevrons, file icons, VCS status colour | selected, focused, modified, ignored, loading | Indentation 12 px per level; no horizontal scrolling above 3 levels |
| Editor tabs | Title, dirty dot, close button, overflow | active, inactive, hover, dirty, error | Middle-click closes; drag reorders; unsaved tabs are visually distinct |
| Editor surface | Gutter, text, overlays | caret, selection, search hit, current line, breakpoint, folded | See [Editor surface](#editor-surface) |
| Breadcrumb | Path segments, dropdown per segment | hover, focused | Hidden if it costs more than one text line of value |
| Tool window | Title bar, actions, content | collapsed, expanded, focused, pinned | Title bar holds actions, not a menu of menus |
| Tabs strip (bottom) | Tabs, badge counts, close | active, inactive, has-errors | Keep tab count under ~6; overflow into a list |
| Status bar | Segments, separators, progress | idle, busy, error | Non-interactive segments look non-interactive |
| Command palette | Input, grouped results, shortcut hints | empty, filtered, no results | Fuzzy match; Enter runs the top result |
| Dialog | Title, body, actions, close | default, busy, error | Primary action right-aligned; Esc cancels; Enter confirms only when safe |
| Notification | Severity stripe, message, actions, dismiss | info, warning, error, progress | Never steal focus; errors link to the failing action |
| Empty state | Icon, one line of intent, primary action | — | Explain what would appear here and how to make it appear |

### Editor surface

| Element | Specification |
|---|---|
| Gutter | Right-aligned line numbers, muted; width derived from the largest visible number, not fixed |
| Caret | 2 px wide, full line height, blink 530 ms, stops blinking while typing |
| Selection | `selection_editor` background, `selection_text` foreground; unfocused surface uses `selection_inactive` |
| Current line | One step lighter than `surface_editor`, no border |
| Indent guides | `border_subtle`, 1 px, aligned to tab stops |
| Diagnostics | 1 px wavy underline in severity colour + gutter marker; never colour-only (icon or tooltip too) |
| Matching bracket | 2 px outline, no fill |
| Fold marker | In the gutter, visible on hover and always when folded |
| Inlay hints | 12 px `text_muted`, hideable, never inside the caret's line box |
| Long lines | Horizontal scroll by default; soft wrap is a per-document setting |
| Minimap / scrollbar | Scrollbar always; minimap off by default |

## Interaction patterns

- **Focus is visible.** Every focusable surface draws a 2 px focus ring in `accent`. Focus follows
  the keyboard, not the mouse, when both are used.
- **One action, one command.** Menu items, shortcuts, palette entries, and tooltips read from the
  same registry. A shortcut that exists only as a raw key handler is a defect.
- **Predictable navigation.** `Esc` closes the innermost transient surface. `Tab`/`Shift+Tab` move
  within a focus scope; `Ctrl+Tab` cycles editor tabs.
- **Context menus are local.** They contain actions for the item under the pointer and nothing else.
- **Hover never mutates state.** Tooltips are informational; nothing opens on hover alone.
- **Destructive actions confirm, and say what will be lost.** Undoable actions do not confirm.
- **Progress is ambient.** Long work shows a determinate bar when progress is known, indeterminate
  otherwise, and can be cancelled from where it started.
- **Errors are actionable.** Every error message names the command, the object, and the next step.
- **Search is first-class.** A single palette for commands, files, and symbols; results grouped and
  keyboard-navigable.
- **Locale changes are live.** Selecting a language re-renders every surface, including the status
  message already on screen, without a restart.

## States

Every surface must define, before review:

| State | Requirement |
|---|---|
| Empty | Explains what would be here and offers the primary action |
| Loading | Named progress, no layout jump; cached content stays visible |
| Error | Message, cause, retry or open-log action; no silence |
| Partial | Stale data marked as stale, with the time it was produced |
| Disabled | Visually distinct, with a reason on hover |
| Very long content | Ellipsis, horizontal scroll, or wrapping — chosen deliberately |
| Narrow window | Panels collapse in a defined order; the editor keeps priority |

## Accessibility

- Contrast: 4.5:1 for body text, 3:1 for large text and essential UI boundaries, in both themes.
  `text_disabled` is exempt as an inactive control; decorative separators use `border_subtle` or
  `border`, and essential boundaries use `border_strong`.
- Colour is never the only carrier of meaning; pair it with an icon, underline, or text.
- Full keyboard operation for every workflow, including dialogs, tabs, trees, and notifications.
- Focus order matches visual order; focus is never trapped without an escape.
- Respect the OS scale factor and reduced-motion preference; no animation is required to understand
  state.
- Screen-reader integration is `Unknown`; document the chosen approach in
  [`architecture.md`](architecture.md) › Open architectural questions before claiming support.

## Motion

Motion communicates change, not decoration.

- 120 ms for hover, focus, and expansion; 180 ms for panel and dialog transitions.
- No decorative animation on the caret, scroll, or keystroke echo. The caret blink is a state
  indicator, not decoration; scrolling and typing never animate.
- Never block input while animating; state is authoritative, animation is a render of it.
- Disable non-essential motion when the OS asks for reduced motion.

## IntelliJ-inspired interaction patterns

Where supported by the roadmap:

- Project navigation, editor tabs, and tool panels stay predictable in position.
- Actions live in menus, shortcuts, and the palette simultaneously, with identical labels.
- Selection, focus, modified documents, and active panels are visually unambiguous.
- Diagnostics are visible without hiding editor context (gutter, underline, status bar counts).
- Tool windows share the bottom edge and keep their tab order stable across sessions.

Do not imitate a feature visually if the underlying behaviour is not implemented.

## Visual verification checklist

- [ ] Shared tokens are used consistently; no new raw colours or sizes in widgets.
- [ ] No user-facing English literal appears in widget code; every label, tooltip, and message
      resolves through the localisation catalog.
- [ ] Switching the language re-renders menus, prompts, and the current status message without a
      restart.
- [ ] Text and controls meet the contrast targets in both themes.
- [ ] Keyboard focus is visible and reachable on every interactive element.
- [ ] Menus, context menus, palette, and shortcuts trigger the same commands.
- [ ] Panels behave sensibly at minimum and maximum sizes, and in a narrow window.
- [ ] Empty, loading, error, partial, and disabled states are understandable.
- [ ] Layout holds at 100%, 125%, and 150% scale.
- [ ] No proprietary IntelliJ assets or copied source code are included.
- [ ] Screenshots reflect the current build and were produced by the documented scenario.

## Verification

1. The theme module is `crates/kaffein-ui/src/theme.rs`; its values were mirrored into this document
   on 2026-10-26. Continue with steps 2–6 before marking the visual design `Verified`.
2. Inventory widgets against the Component table; list missing components and missing states.
3. Re-measure contrast for each text/background pairing in both themes against the shipped values
   and record failing pairs; the baseline was computed on 2026-10-26 (see [Design tokens](#design-tokens)).
4. Capture the layout at three window sizes and three scale factors; link the images.
5. Walk one workflow keyboard-only and list every focus stop that is invisible or unreachable.
6. Date the review. Until step 1 completes, every value here is `Proposed`.
7. Launch with both bundled locales, switch language from **View › Language**, and confirm menus,
   prompts, the status message, and the status bar locale name update without a restart; record the
   date and platform.
