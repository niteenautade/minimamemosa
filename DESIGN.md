# MinimaMemosa — UI Theme & Styling Design

> **Stack:** Tailwind CSS (Play CDN, v3.x) + CSS Custom Properties (OKLCH) + HTMX + highlight.js  
> **Dark Mode:** Client-side class-based toggle with `localStorage` persistence

---

## Table of Contents

1. [Color System](#1-color-system)
2. [Dark / Light Mode](#2-dark--light-mode)
3. [Typography](#3-typography)
4. [Tailwind Configuration](#4-tailwind-configuration)
5. [Layout & Component Styling](#5-layout--component-styling)
6. [Animations & Transitions](#6-animations--transitions)
7. [Highlight.js Syntax Themes](#7-highlightjs-syntax-themes)
8. [Custom Utilities](#8-custom-utilities)
9. [Component-Specific Styles](#9-component-specific-styles)

---

## 1. Color System

All colors use the **OKLCH** color space for perceptual uniformity. Colors are defined as CSS custom properties on `:root` (light) and `.dark` (dark), then mapped to Tailwind utility classes via the config.

### 1.1 CSS Custom Properties

| Token | Light Mode | Dark Mode | Visual Description |
|-------|-----------|-----------|--------------------|
| `--bg` | `oklch(0.9818 0.0054 95.0986)` | `oklch(0.24 0.008 255)` | Page background (warm off-white / dark blue-gray) |
| `--fg` | `oklch(0.2438 0.0269 95.7226)` | `oklch(0.9 0.006 255)` | Primary text (near-black warm / off-white) |
| `--card` | `oklch(1 0 0)` | `oklch(0.275 0.009 255)` | Card/surface background (white / dark card) |
| `--card-fg` | `oklch(0.1908 0.002 106.5859)` | `oklch(0.9 0.006 255)` | Card text |
| `--border` | `oklch(0.8847 0.0069 97.3627)` | `oklch(0.38 0.01 255)` | Borders & dividers (light warm gray) |
| `--muted` | `oklch(0.9341 0.0153 90.239)` | `oklch(0.35 0.011 255)` | Muted backgrounds (warm light gray) |
| `--muted-fg` | `oklch(0.5559 0.0075 97.4233)` | `oklch(0.72 0.007 255)` | Muted text (medium warm gray) |
| `--sidebar` | `oklch(0.9663 0.008 98.8792)` | `oklch(0.21 0.009 255)` | Sidebar background (warm off-white) |
| `--sidebar-fg` | `oklch(0.359 0.0051 106.6524)` | `oklch(0.76 0.007 255)` | Sidebar text |

### 1.2 Accent Colors

| Usage | Light | Dark |
|-------|-------|------|
| Links, focus rings, accents | `oklch(0.45 0.08 250)` | `oklch(0.62 0.11 250)` |

These are applied **inline** via `.dark`-scoped selectors, not as CSS variables.

### 1.3 Hardcoded Hex / Named Colors

| Value | Usage |
|-------|-------|
| `#8e8e8a` | Secondary metadata text, inactive delete icon |
| `bg-blue-50/100/500/600/700` | Primary button, badges, active indicators, focus rings |
| `text-gray-400/500` | Side text, login/register page labels |
| `#e5e5e0` | Calendar day hover (light mode) |
| `#3e4045` | Calendar day hover, resource thumbnail backgrounds (dark mode) |
| `#f0f0eb` | Resource thumbnail backgrounds (light mode) |
| `text-green-600/700 / text-green-400/500` | Public visibility indicators |
| `text-amber-600/700 / text-amber-400/500` | Protected visibility indicators |
| `text-red-500` | Delete hover states, error text |
| `bg-red-50` / `bg-red-900/20` | Delete button hover backgrounds |
| `bg-amber-50` / `bg-amber-900/20` | Protected badges and icons |
| `bg-green-50` / `bg-green-900/20` | Public badges and icons |

### 1.4 Blue Accent Semantic Mapping

| Tailwind Class | Context |
|----------------|---------|
| `bg-blue-600 hover:bg-blue-700` | Primary buttons (Sign In, Register, Save, Set Password) |
| `text-blue-600 dark:text-blue-400` | Tags, active icons, link text |
| `bg-blue-50 dark:bg-blue-900/20` | Tags, badges, highlight backgrounds |
| `bg-blue-100 dark:bg-blue-900/30` | Active nav icon highlight |
| `focus:ring-blue-500` | All focus rings |
| `bg-blue-500/70 dark:bg-blue-400/50` | Calendar days with memos |
| `bg-blue-500 dark:bg-blue-400` | Calendar "today" indicator |

---

## 2. Dark / Light Mode

### 2.1 Mechanism

- **Strategy:** Class-based dark mode via `dark` class on `<html>`.
- **Init:** Checks `localStorage.getItem('theme')`, falls back to `prefers-color-scheme: dark`.
- **Toggle:** `toggleTheme()` function toggles `dark` class and saves preference to `localStorage`.
- **Zero server overhead:** Entirely client-side; the server never knows the current theme.

### 2.2 Theme Persistence

```
Key:     "theme"
Values:  "dark" | "light"
Storage: localStorage (browser)
```

### 2.3 Syntax Highlighting Theme Toggle

The `toggleTheme()` function also swaps the highlight.js stylesheet via a `<link>` element with `id="hljs-theme"`:

| Theme State | Stylesheet |
|-------------|-----------|
| Dark | `/static/github-dark-dimmed.min.css` |
| Light | `/static/github.min.css` |

A `DOMContentLoaded` handler ensures the correct stylesheet is active on page load.

---

## 3. Typography

### 3.1 Font Families

```css
--font-sans: ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI",
             Roboto, "Helvetica Neue", Arial, "Noto Sans", sans-serif;

--font-mono: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas,
             "Liberation Mono", "Courier New", monospace;
```

- Body: `sans` (native system font stack)
- Code blocks / inline code: `mono`
- Applied via `body { font-family: var(--font-sans, ...) }` and Tailwind's `font-sans`/`font-mono` classes.

### 3.2 Font Sizes

| Context | Tailwind Class | Actual `font-size` |
|---------|---------------|-------------------|
| Page title | `text-2xl` | `1.5rem` |
| Note titles | `text-sm font-medium` | `0.875rem` |
| Memo content | `text-[15px]` | `15px` |
| Tags / badges | `text-[9px]` / `text-[10px]` | `9px` / `10px` |
| Metadata / timestamps | `text-xs` | `0.75rem` |
| Editor placeholders | `text-base` | `1rem` |
| Sidebar headings | `text-xs font-semibold uppercase tracking-wider` | `0.75rem` uppercase |
| Tip text ("Press / for commands") | `text-[10px]` | `10px` |
| Modal title | `text-lg font-semibold` | `1.125rem` |
| Toast messages | `text-sm font-medium` | `0.875rem` |

### 3.3 Line Heights

| Context | `line-height` |
|---------|--------------|
| Memo content (`.memo-content p`) | `1.5` |
| Editor (`.tiptap-editor p`, `.ProseMirror p`) | `1.5` |
| Editor headings | browser default |
| Save button state | `leading-none` |

---

## 4. Tailwind Configuration

Defined inline via `tailwind.config` before the CDN script executes.

```javascript
tailwind.config = {
    darkMode: 'class',
    theme: {
        extend: {
            fontFamily: {
                sans: ['ui-sans-serif', 'system-ui', ...],
                mono: ['ui-monospace', 'SFMono-Regular', ...],
            },
            colors: {
                background: 'var(--bg)',
                foreground: 'var(--fg)',
                card: 'var(--card)',
                'card-fg': 'var(--card-fg)',
                border: 'var(--border)',
                muted: 'var(--muted)',
                'muted-fg': 'var(--muted-fg)',
                sidebar: 'var(--sidebar)',
                'sidebar-fg': 'var(--sidebar-fg)',
            }
        }
    }
}
```

All custom color tokens are mapped to CSS variables, enabling automatic light/dark switching when the `dark` class toggles on `<html>`.

---

## 5. Layout & Component Styling

### 5.1 Page Shell

```
body:  bg-background text-foreground min-h-screen
```

### 5.2 Header

```
bg-white dark:bg-gray-900
border-b border-border
px-6 py-2.5
```

Exception for header: uses hardcoded `bg-white`/`dark:bg-gray-900` instead of `bg-card` for visual separation.

### 5.3 Sidebar / Panels

All side panels (timeline sidebar, notes panel, resources panel):

```
w-72
bg-sidebar text-sidebar-fg
border-r border-border
```

Icon bar (the narrow vertical strip of nav icons):

```
w-14
bg-card
border-r border-border
```

### 5.4 Icon Navigation Icons

| State | Classes |
|-------|---------|
| Inactive | `text-muted-fg hover:bg-muted hover:text-foreground` |
| Active | `bg-blue-100 dark:bg-blue-900/30 text-blue-600 dark:text-blue-400` |

### 5.5 Cards & Surfaces

Memo cards in timeline:

```
p-4
bg-card
rounded-xl
border border-border
shadow-sm hover:shadow-md
transition-shadow
```

Editor container:

```
bg-white dark:bg-gray-900
rounded-lg
border border-gray-200 dark:border-gray-700
shadow-sm
```

Exception: editor uses `bg-white`/`dark:bg-gray-900` and `border-gray-200`/`dark:border-gray-700` instead of custom variables for visual distinction.

### 5.6 Modals

| Element | Styling |
|---------|---------|
| Overlay | `rgba(0,0,0,0.5)` backdrop |
| Modal body | `bg-card rounded-xl border border-border shadow-xl p-6 w-full max-w-sm` |
| Close button | `bg-black/20 hover:bg-black/40 text-white/70 hover:text-white rounded-full` |
| Image modal | `background: rgba(0,0,0,0.85)` |

### 5.7 Buttons

| Type | Classes |
|------|---------|
| Primary | `bg-blue-600 hover:bg-blue-700 text-white font-medium rounded-lg` |
| Secondary / Cancel | `bg-transparent hover:bg-muted text-foreground` |
| Muted | `bg-muted hover:bg-muted/80 text-muted-fg` |
| Icon | `p-1 rounded-md text-muted-fg hover:text-foreground hover:bg-muted` |
| Delete | `text-[#8e8e8a] hover:text-red-500 hover:bg-red-50 dark:hover:bg-red-900/20` |
| Disabled primary | `disabled:opacity-50 disabled:cursor-not-allowed disabled:hover:bg-blue-600` |

### 5.8 Form Inputs

```css
w-full px-3 py-2
border border-border  /* or border-gray-300 dark:border-gray-600 on login/register */
rounded-lg
bg-card
focus:outline-none focus:ring-2 focus:ring-blue-500
```

Login/register pages use `border-gray-300 dark:border-gray-600` for input borders.

### 5.9 Visibility Badges

| Visibility | Classes |
|------------|---------|
| Public | `bg-green-50 dark:bg-green-900/20 text-green-700 dark:text-green-400 border border-green-100 dark:border-green-800` |
| Protected | `bg-amber-50 dark:bg-amber-900/20 text-amber-700 dark:text-amber-400 border border-amber-100 dark:border-amber-800` |

### 5.10 Tags

```css
inline-block px-1.5 py-0.5 text-[9px] font-medium rounded
bg-blue-50 dark:bg-blue-900/20 text-blue-600 dark:text-blue-400
```

---

## 6. Animations & Transitions

### 6.1 Theme Toggle

```css
transition-colors
```

Applied to all buttons, icons, hover targets, and theme-sensitive elements.

### 6.2 Editor Focus

```css
.memo-editor {
    transition: border-color 0.15s ease;
}
.memo-editor:focus-within {
    border-color: oklch(0.45 0.08 250); /* light mode */
}
.dark .memo-editor:focus-within {
    border-color: oklch(0.62 0.11 250); /* dark mode */
}
```

### 6.3 Memo Card Hover

```css
transition-shadow
```

Cards go from `shadow-sm` to `shadow-md` on hover.

### 6.4 Action Buttons Visibility

Memo action buttons (share, edit, delete) are initially hidden and appear on parent hover:

```css
.group/memo:hover .group-hover/memo:opacity-100
```

### 6.5 Loading Shimmer

```css
@keyframes shimmer {
    0% { background-position: -200% 0; }
    100% { background-position: 200% 0; }
}
.shimmer-bg {
    background: linear-gradient(90deg, var(--muted) 25%, var(--border) 50%, var(--muted) 75%);
    background-size: 200% 100%;
    animation: shimmer 1.5s infinite linear;
}
```

### 6.6 Global Loader

A fixed overlay with backdrop blur and SVG bar-chart animation, triggered by HTMX events:

```css
#global-loader {
    position: fixed; inset: 0; z-index: 99999; display: none;
    align-items: center; justify-content: center;
    background: rgba(0,0,0,0.3); backdrop-filter: blur(2px);
}
#global-loader.active { display: flex; }
```

Shown on `htmx:beforeSend`, hidden on `htmx:afterSettle` / `htmx:responseError` / `htmx:sendError`.

### 6.7 Toast

Bottom-center notification with slide + fade transition:

```css
transition-all duration-300
opacity-0 translate-y-2  →  opacity-100 translate-y-0
```

- Error: `bg-red-600 text-white`
- Success: `bg-green-600 text-white`
- Info: `bg-gray-800 dark:bg-gray-200 text-white dark:text-gray-900`

Auto-dismisses after 2500ms.

### 6.8 Image Resize Menu

```css
#image-resize-menu {
    background: var(--card);
    border: 1px solid var(--border);
    border-radius: 0.5rem;
    box-shadow: 0 4px 12px rgba(0,0,0,0.15);
    z-index: 9999;
}
#image-resize-menu button:hover { background: var(--muted); }
```

---

## 7. Highlight.js Syntax Themes

Two highlight.js CSS themes are inlined in the template:

| Theme | File | Used For |
|-------|------|----------|
| GitHub (light) | `/static/github.min.css` | Light mode code blocks |
| GitHub Dark Dimmed | `/static/github-dark-dimmed.min.css` | Dark mode code blocks |

The stylesheet link element has `id="hljs-theme"` and is swapped dynamically on theme toggle and page load.

---

## 8. Custom Utilities

### 8.1 Avatar Initials

```css
.avatar-initials {
    display: inline-flex; align-items: center; justify-content: center;
    width: 2rem; height: 2rem; border-radius: 9999px;
    font-size: 0.875rem; font-weight: 600; flex-shrink: 0;
}
.avatar-initials-sm {
    width: 1.5rem; height: 1.5rem; font-size: 0.75rem;
}
```

### 8.2 Auto-expand Textarea

```css
.auto-expand-textarea {
    min-height: 2.5rem; overflow: hidden; resize: none;
}
```

### 8.3 Scrollbar

```css
* {
    scrollbar-width: thin;
    scrollbar-color: color-mix(in srgb, var(--muted-fg) 45%, transparent) transparent;
}
```

Thin, muted scrollbar globally.

---

## 9. Component-Specific Styles

### 9.1 Memo Content (Rendered Markdown)

| Element | Styling |
|---------|---------|
| `h1` | `font-size: 1.5rem; font-weight: 700; border-bottom: 1px solid var(--border); padding-bottom: 0.25rem; margin-bottom: 0.5rem;` |
| `h2` | `font-size: 1.25rem; font-weight: 600; margin-bottom: 0.5rem;` |
| `h3` | `font-size: 1.125rem; font-weight: 600; margin-bottom: 0.25rem;` |
| `p` | `margin-bottom: 0; line-height: 1.5;` |
| `p:last-child` | `margin-bottom: 0;` |
| `ul, ol` | `padding-left: 1.5rem; margin-bottom: 0.5rem;` |
| `li` | `list-style: disc; margin-bottom: 0.25rem;` |
| `ol li` | `list-style: decimal;` |
| `code` (inline) | `background: var(--muted); padding: 0.125rem 0.375rem; border-radius: 0.25rem; font-size: 0.875rem; color: var(--fg); font-family: mono;` |
| `pre` | `background: var(--muted); padding: 0.75rem; border-radius: 0.5rem; overflow-x: auto; margin-bottom: 0.75rem; border: 1px solid var(--border);` |
| `pre code` | `background: none; padding: 0; color: inherit;` |
| `blockquote` | `border-left: 3px solid var(--border); padding-left: 0.75rem; margin: 0.5rem 0; color: var(--muted-fg);` |
| `a` | `color: oklch(0.45 0.08 250); text-decoration: none;` (light) |
| `a:hover` | `text-decoration: underline;` |
| `.dark a` | `color: oklch(0.62 0.11 250);` |
| `hr` | `border: none; border-top: 1px solid var(--border); margin: 0.75rem 0;` |
| `table` | `border-collapse: collapse; margin-bottom: 0.5rem; width: 100%;` |
| `th, td` | `border: 1px solid var(--border); padding: 0.375rem 0.75rem; text-align: left;` |
| `th` | `background: var(--muted); font-weight: 600;` |
| `img` | `max-width: 100%; border-radius: 0.5rem; margin-top: 0.5rem; margin-bottom: 0.5rem;` |
| `ul.task-list` | `list-style: none; padding-left: 0;` |
| `li.task-list-item` | `display: flex; align-items: flex-start; gap: 0.375rem;` |

### 9.2 Tiptap Editor

| Element | Styling |
|---------|---------|
| `.ProseMirror` | `outline: none; white-space: pre-wrap; word-wrap: break-word; min-height: 4rem;` |
| `.ProseMirror p` | `margin: 0.25rem 0;` |
| `.ProseMirror h1` | `font-size: 1.5rem; font-weight: 700; margin: 0.25rem 0; border-bottom: 1px solid var(--border); padding-bottom: 0.25rem;` |
| `.ProseMirror h2` | `font-size: 1.25rem; font-weight: 600; margin: 0.25rem 0;` |
| `.ProseMirror h3` | `font-size: 1.125rem; font-weight: 600; margin: 0.25rem 0;` |
| `.ProseMirror ul, ol` | `padding-left: 1.25rem; margin: 0.25rem 0;` |
| `.ProseMirror li` | `list-style: disc;` |
| `.ProseMirror ol li` | `list-style: decimal;` |
| `.ProseMirror code` | `background: var(--muted); padding: 0.125rem 0.25rem; border-radius: 0.25rem; font-size: 0.875rem;` |
| `.ProseMirror pre` | `background: var(--muted); padding: 0.5rem; border-radius: 0.375rem; margin: 0.25rem 0;` |
| `.ProseMirror pre code` | `background: none; padding: 0;` |
| `.ProseMirror blockquote` | `border-left: 2px solid var(--border); padding-left: 0.5rem; margin: 0.25rem 0; color: var(--muted-fg);` |
| `.ProseMirror a` | `color: oklch(0.45 0.08 250);` (light) |
| `.dark .ProseMirror a` | `color: oklch(0.62 0.11 250);` |
| `.ProseMirror hr` | `border: none; border-top: 1px solid var(--border); margin: 0.75rem 0;` |
| `.ProseMirror table` | `border-collapse: collapse; margin: 0.25rem 0; width: 100%;` |
| `.ProseMirror th, td` | `border: 1px solid var(--border); padding: 0.25rem 0.5rem; text-align: left;` |
| `.ProseMirror th` | `background: var(--muted); font-weight: 600;` |
| `.ProseMirror img` | `max-width: 100%; height: auto; border-radius: 0.5rem; margin-top: 0.5rem; margin-bottom: 0.5rem; cursor: pointer;` |
| `.ProseMirror p.is-editor-empty:first-child::before` | Placeholder pseudo-element, `color: var(--muted-fg)` |

### 9.3 Slash Commands Menu

```css
#slash-menu {
    min-width: 260px;
}
```
Command items: `flex items-center gap-2 w-full px-3 py-1.5 text-xs text-foreground`.  
Selected item: `bg-muted text-foreground`.

### 9.4 Calendar

- Day headers: `text-[10px] text-muted-fg font-medium`
- Days with memos: `bg-blue-500/70 dark:bg-blue-400/50 text-white font-medium hover:bg-blue-500 dark:hover:bg-blue-400/70`
- Today: `bg-blue-500 dark:bg-blue-400 text-white font-semibold shadow-sm` with `ring-1 ring-inset ring-white/30`
- Empty days: `text-muted-fg hover:bg-[#e5e5e0] dark:hover:bg-[#3e4045]`
- Date grid: `grid-cols-7 gap-0.5`, days as `aspect-square text-[11px]`

### 9.5 Resource Thumbnails

```
w-10 h-10 (or w-12 h-12 in note list)
rounded-lg overflow-hidden
bg-[#f0f0eb] dark:bg-[#3e4045]
```

### 9.6 Emoji Picker

```
z-50 w-[280px] max-h-[200px] overflow-y-auto
bg-card border border-border rounded-xl shadow-xl p-2
Emoji grid: grid-cols-7 gap-0.5 text-lg
```

### 9.7 Link Memo Dropdown

```
z-50 w-[250px] bg-card border border-border rounded-xl shadow-xl
Search input: bg-muted border border-border rounded-lg
```

### 9.8 Protected Note Password Page

- Lock icon in `bg-amber-50 dark:bg-amber-900/20 text-amber-600 dark:text-amber-400 border border-amber-100 dark:border-amber-800`
- Card: `bg-card rounded-xl border border-border shadow-md p-6 w-full max-w-sm`

---

## File Reference

All theme styling is defined in a single source file:

| File | Contents |
|------|----------|
| `src/templates.rs` | All HTML templates, inline `<style>` block (lines 81-186), Tailwind config (lines 12-33), theme toggle JS (lines 40-57), all Tailwind classes across 10 templates |
| `static/tailwindcss.js` | Self-hosted Tailwind Play CDN build (v3.x) |
| `static/github.min.css` | Light syntax highlighting theme |
| `static/github-dark-dimmed.min.css` | Dark syntax highlighting theme |
