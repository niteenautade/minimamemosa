# MinimaMemosa — Unified Notes + Timeline Design

> **Goal:** Merge the separate `/app/timeline` and `/app/notes` views into a single, unified experience that feels intuitive for everyday users on both desktop and mobile.  
> **Principle:** One place for everything. Users should never need to switch between two views to find or interact with their notes.  
> **Constraint:** Zero new server-side routes required. All existing HTMX endpoints reused. No RAM impact.

---

## Table of Contents

1. [Problem with the Current Design](#1-problem-with-the-current-design)
2. [Unified Design Philosophy](#2-unified-design-philosophy)
3. [Unified Layout — Desktop](#3-unified-layout--desktop)
4. [Unified Layout — Mobile](#4-unified-layout--mobile)
5. [Unified Sidebar Design](#5-unified-sidebar-design)
6. [Main Content Area Behavior](#6-main-content-area-behavior)
7. [Navigation Simplification](#7-navigation-simplification)
8. [Interaction Flows](#8-interaction-flows)
9. [Implementation Plan — Templates](#9-implementation-plan--templates)
10. [Implementation Plan — Rust Routes](#10-implementation-plan--rust-routes)
11. [Implementation Plan — JavaScript](#11-implementation-plan--javascript)
12. [Features Preserved](#12-features-preserved)
13. [UX Improvements Summary](#13-ux-improvements-summary)

---

## 1. Problem with the Current Design

The current app has **two separate views** for essentially the same data:

| View | Sidebar Shows | Main Area Shows | URL |
|:---|:---|:---|:---|
| **Timeline** | Search, Calendar, Tags | Editor + chronological feed | `/app/timeline` |
| **Notes** | Searchable note list with thumbnails | Single selected note | `/app/notes` |

**User pain points:**

1. **Mental model split** — "Where do I go to find a note?" Users must decide between Timeline (browse chronologically) or Notes (browse by title list). This is unnecessary cognitive overhead.
2. **Context loss** — Switching from Timeline to Notes is a full page navigation. Users lose their scroll position, any in-progress editor state, and their mental context.
3. **Duplicated search** — Both views have a search bar that does similar things, but they behave slightly differently (one filters the sidebar list, the other searches the timeline).
4. **Feature fragmentation** — Calendar and tags are only available in the Timeline view. The note list with thumbnails and titles is only available in the Notes view. Users can't use both at once.

---

## 2. Unified Design Philosophy

### The "Notes App" Mental Model

The best note-taking apps (Apple Notes, Notion, Obsidian) all use the same pattern:

```
┌──────────┬────────────────────────────────┐
│ Note List │  Note Content / Editor         │
│ (sidebar) │                                │
│           │                                │
│  [Search] │                                │
│  Note A ← │  ← Active note shown here     │
│  Note B   │                                │
│  Note C   │                                │
│  ...      │                                │
└──────────┴────────────────────────────────┘
```

Users intuitively understand: **left = browse, right = read/write**.

### Our Unified Design

We keep MinimaMemosa's unique strengths (timeline grouping, calendar, tags, quick memo posting) but merge them into a **single, cohesive view**:

- **One sidebar** that contains the note list, search, calendar, and tags — all in one panel
- **One main area** that shows the editor + timeline feed by default, or a single focused note when selected from the sidebar
- **One URL** (`/app`) — no more switching between pages

### Key Principles

1. **Default = Timeline feed** — When the user opens the app, they see the editor + chronological feed (exactly as today). This is the "home" state.
2. **Click a note in the sidebar = Focus that note** — The main area swaps to show just that note (with edit/delete/share actions). A clear "← Back to all notes" button returns to the feed.
3. **Search is universal** — One search bar filters both the sidebar note list AND the timeline feed simultaneously.
4. **Calendar & Tags are always accessible** — Collapsible sections in the sidebar, visible alongside the note list.

---

## 3. Unified Layout — Desktop

### Layout Structure (at `lg:` breakpoint, 1024px+)

```
┌─────────────────────────────────────────────────────────────┐
│  Header: [Logo]  ─────────────────  [🌙] [Avatar] [Logout]  │
├────┬─────────────────┬──────────────────────────────────────┤
│    │  ┌─ Search ──┐  │                                      │
│ 📝 │  │           │  │  ┌── Editor ──────────────────────┐  │
│    │  └───────────┘  │  │  What's on your mind...        │  │
│ 🖼  │  ┌─ Notes ──┐  │  │                          [Save]│  │
│    │  │ Note A    │◄─│──│──── Main Content ──────────────│  │
│ ⚙  │  │ Note B    │  │  │                                │  │
│    │  │ Note C    │  │  │  TODAY                          │  │
│    │  │ Note D    │  │  │  ┌─ Memo Card ──────────┐      │  │
│    │  │ ...       │  │  │  │ Content...           │      │  │
│    │  └───────────┘  │  │  └──────────────────────┘      │  │
│    │  ▾ Calendar     │  │                                │  │
│    │  ▾ Tags         │  │  YESTERDAY                     │  │
│    │                 │  │  ┌─ Memo Card ──────────┐      │  │
│    │                 │  │  │ Content...           │      │  │
│    │                 │  │  └──────────────────────┘      │  │
├────┴─────────────────┴──┴────────────────────────────────┘  │
```

### Key Changes from Current

| Component | Before | After |
|:---|:---|:---|
| Icon bar | 3 icons: Timeline, Notes, Resources | 2 icons: **Notes** (unified), Resources |
| Sidebar | Two separate sidebars (timeline vs notes) | **One unified sidebar** (always visible) |
| Main area | Changes based on active_panel | Single behavior: feed or focused note |
| `/app/notes` route | Separate page load | **Removed** (redirects to `/app`) |
| `/app/timeline` route | Separate page load | Becomes `/app` (or kept as alias) |

---

## 4. Unified Layout — Mobile

### Mobile Layout (< 1024px)

```
┌─────────────────────────────┐
│  [☰] MinimaMemosa  [🌙][👤] │
├─────────────────────────────┤
│                             │
│  ┌── Editor ─────────────┐  │
│  │ What's on your mind...│  │
│  │                 [Save]│  │
│  └───────────────────────┘  │
│                             │
│  TODAY                      │
│  ┌── Memo Card ──────────┐  │
│  │ Content...            │  │
│  └───────────────────────┘  │
│                             │
│  YESTERDAY                  │
│  ┌── Memo Card ──────────┐  │
│  │ Content...            │  │
│  └───────────────────────┘  │
│                             │
├─────────────────────────────┤
│  [📝 Notes]  [🖼 Resources] │  ← Bottom nav (simplified)
└─────────────────────────────┘

 ☰ Hamburger opens:
 ┌────────────────────┐
 │ Search             │
 │ ─────────────────  │
 │ Note A             │
 │ Note B             │
 │ Note C             │
 │ ...                │
 │ ─────────────────  │
 │ ▸ Calendar         │
 │ ▸ Tags             │
 └────────────────────┘
```

### Mobile Bottom Nav — Simplified

Since Timeline and Notes are merged, the bottom nav goes from 4 items to 3:

| Before | After |
|:---|:---|
| Timeline · Notes · Resources · Settings | **Notes** · Resources · Settings |

This is cleaner and more thumb-friendly.

---

## 5. Unified Sidebar Design

The unified sidebar combines the best of both the timeline sidebar and the notes panel into a single, well-organized panel.

### Sidebar Section Layout (top to bottom)

```
┌──────────────────────────┐
│  NOTES                   │ ← Section header
├──────────────────────────┤
│  🔍 Search notes...      │ ← Universal search
├──────────────────────────┤
│                          │
│  📄 My first note        │ ← Note list (scrollable)
│     Jun 24 · #journal    │
│                          │
│  📄 Project ideas     🖼 │ ← Thumbnail for image notes
│     Jun 23 · #work       │
│                          │
│  📄 Meeting notes        │
│     Jun 22               │
│                          │
│  📄 Shopping list        │
│     Jun 21 · #personal   │
│                          │
│  ... (infinite scroll)   │
│                          │
├──────────────────────────┤
│  ▸ Calendar     June '26 │ ← Collapsible section
├──────────────────────────┤
│  ▸ Tags            12    │ ← Collapsible section  
└──────────────────────────┘
```

### 5.1 Search Bar

- **Single search bar** at the top of the sidebar.
- **Dual-action:** filters the note list in the sidebar AND triggers an HTMX search on the main timeline (reusing the existing `/search` endpoint).
- Debounced (existing `debouncedFilterSidebar` + HTMX `delay:400ms`).

### 5.2 Note List

- **Always visible** — the scrollable list of all notes is always accessible.
- **Each item shows:** title, date, visibility icon, tags (compact), optional image thumbnail.
- **Active state:** the currently viewed note is highlighted with the accent color.
- **Click behavior:** loads that note as a focused view in the main area.
- **Infinite scroll:** uses existing sentinel-based loading (`/notes-panel?offset=N`).
- **Auto-refresh:** listens for `memoUpdated` events via HTMX to refresh when notes are created/edited/deleted.

### 5.3 Calendar (Collapsible)

- **Default state:** collapsed (just shows "▸ Calendar · June 2026").
- **Expanded:** shows the month grid with clickable days (existing calendar behavior).
- **Click a day:** filters the timeline feed to that date.
- **State persistence:** expand/collapse saved to `localStorage`.

### 5.4 Tags (Collapsible)

- **Default state:** collapsed (shows "▸ Tags · 12" with count).
- **Expanded:** shows the tag cloud (existing tag behavior).
- **Click a tag:** filters the timeline feed to that tag.
- **State persistence:** expand/collapse saved to `localStorage`.

### 5.5 Why Collapsible Calendar + Tags?

- The note list is the **primary navigation** — users browse by title most often.
- Calendar and tags are **secondary filters** — useful but not needed every session.
- Collapsing them gives maximum vertical space to the note list.
- Users who prefer the calendar-first workflow can simply keep it expanded.

---

## 6. Main Content Area Behavior

The main area has **two states**, toggled by user interaction:

### State 1: Timeline Feed (Default)

This is what users see when they open the app or click "← Back to all notes."

```
┌─────────────────────────────────────────┐
│  ┌── Editor ──────────────────────────┐ │
│  │  What's on your mind...            │ │
│  │  [😀] [+] [Private ▾]       [Save] │ │
│  └────────────────────────────────────┘ │
│                                         │
│  TODAY ──────────────────────────── (3)  │
│  ┌── Memo Card ──────────────────────┐  │
│  │  username · 2h ago        [✏][🗑]  │  │
│  │  Note content here...             │  │
│  │  #tag1 #tag2                      │  │
│  └───────────────────────────────────┘  │
│  ┌── Memo Card ──────────────────────┐  │
│  │  ...                              │  │
│  └───────────────────────────────────┘  │
│                                         │
│  YESTERDAY ─────────────────────── (1)  │
│  ┌── Memo Card ──────────────────────┐  │
│  │  ...                              │  │
│  └───────────────────────────────────┘  │
│                                         │
│  (infinite scroll loads more)           │
└─────────────────────────────────────────┘
```

**Identical to today's timeline view.** No changes to the editor, memo cards, date groups, or infinite scroll.

### State 2: Focused Note (When Note Selected from Sidebar)

When the user clicks a note in the sidebar, the main area transitions to show **only that note**:

```
┌─────────────────────────────────────────┐
│  ← Back to all notes                    │ ← Click to return to State 1
│                                         │
│  ┌── Memo Card (focused) ────────────┐  │
│  │  username · Jun 24, 2:30 PM       │  │
│  │  [Share] [Edit] [Delete]          │  │
│  │                                   │  │
│  │  Note content rendered here...    │  │
│  │                                   │  │
│  │  #tag1 #tag2                      │  │
│  └───────────────────────────────────┘  │
│                                         │
│  (editor is hidden in this state)       │
└─────────────────────────────────────────┘
```

**Key UX details:**

1. The **editor is hidden** in focused mode — the user is reading, not composing. This reduces visual clutter.
2. The "← Back to all notes" link is prominent at the top. Clicking it restores the full timeline feed.
3. The focused note uses the same `memo_fragment` template — actions (edit, delete, share) are inline.
4. The sidebar highlights the active note with accent color.
5. Editing a focused note works exactly as today (inline edit form replaces the card).

### State Transitions

```
                    ┌──────────────────────┐
  App opens ───────►│  State 1: Feed       │◄──── "← Back" button
                    │  (editor + timeline) │◄──── Search/filter clears
                    └──────────┬───────────┘
                               │
                    Click note in sidebar
                               │
                    ┌──────────▼───────────┐
                    │  State 2: Focused    │
                    │  (single note view)  │
                    └──────────────────────┘
```

---

## 7. Navigation Simplification

### Desktop Icon Bar

| Before (3 items) | After (2 items) |
|:---|:---|
| 📝 Timeline | 📝 **Notes** (unified) |
| 📄 Notes | *(removed — merged into Notes)* |
| 🖼 Resources | 🖼 Resources |
| ⚙ Settings | ⚙ Settings |

The "Notes" icon becomes the unified view. The "Timeline" icon is removed since it's now the default state of the Notes view.

### Mobile Bottom Nav

| Before (4 items) | After (3 items) |
|:---|:---|
| Timeline · Notes · Resources · Settings | **Notes** · Resources · Settings |

### URL Routes

| Before | After | Behavior |
|:---|:---|:---|
| `/app/timeline` | `/app` | Shows unified view (feed state) |
| `/app/notes` | `/app` | **Redirects to `/app`** (backward compat) |
| `/app/resources` | `/app/resources` | Unchanged |

The `/app/timeline` route can remain as an alias for `/app` for backward compatibility.

---

## 8. Interaction Flows

### Flow 1: User Opens the App

1. User navigates to `/app` (or `/app/timeline`).
2. **Sidebar:** Shows search + note list + collapsed calendar + collapsed tags.
3. **Main area:** Shows editor + chronological timeline feed.
4. **No note is highlighted** in the sidebar (feed mode).

### Flow 2: User Creates a New Note

1. User types in the editor, clicks Save.
2. New memo card prepends to the timeline feed (existing HTMX `afterbegin` swap).
3. `memoUpdated` event fires → sidebar note list refreshes automatically.
4. New note appears at the top of the sidebar list.

### Flow 3: User Clicks a Note in the Sidebar

1. User clicks "Meeting notes" in the sidebar.
2. **Sidebar:** "Meeting notes" gets highlighted (accent background).
3. **Main area:** Transitions to focused view — editor hides, only the selected note's `memo_fragment` is shown, with a "← Back to all notes" link above.
4. **On mobile:** If the sidebar drawer is open, it auto-closes after selection.

### Flow 4: User Clicks "← Back to all notes"

1. User clicks the back link.
2. **Sidebar:** Note highlight is removed.
3. **Main area:** Transitions back to feed — editor reappears, timeline feed reloads.

### Flow 5: User Searches

1. User types in the sidebar search bar.
2. **Sidebar:** Note list filters client-side (existing `filterNotesSidebar` behavior).
3. **Main area:** Timeline feed updates via HTMX search (existing `/search` endpoint).
4. Both update simultaneously, debounced.

### Flow 6: User Clicks a Calendar Day

1. User expands the calendar section, clicks "Jun 15."
2. **Main area:** Timeline feed filters to that date (existing HTMX behavior).
3. **Sidebar:** Note list can optionally filter too (enhancement, not required).

### Flow 7: User Clicks a Tag

1. User expands the tags section, clicks "#journal."
2. **Main area:** Timeline feed filters to that tag (existing HTMX behavior).

### Flow 8: User Edits a Note (from focused view)

1. Note is in focused view. User clicks Edit.
2. Inline edit form appears (existing `editMemo()` behavior).
3. User edits, clicks Save.
4. HTMX swaps the updated `memo_fragment` in place.
5. `memoUpdated` event fires → sidebar refreshes (title/date may have changed).

### Flow 9: User Deletes a Note (from focused view)

1. Note is in focused view. User clicks Delete.
2. Confirmation dialog. User confirms.
3. HTMX deletes the memo, swaps it out.
4. **Main area auto-transitions back to feed** (since there's no note to show).
5. `memoUpdated` event fires → sidebar refreshes.

---

## 9. Implementation Plan — Templates

All changes are in `src/templates.rs`. Here's the section-by-section plan.

### A. Remove the Separate Sidebar Panels → Replace with Unified Sidebar

**Delete / merge these separate panels:**

```
Current: sidebar-panel (timeline sidebar)  ← content merged into unified sidebar
Current: notes-panel (notes list sidebar)  ← content merged into unified sidebar
```

**Replace with a single unified sidebar:**

```html
<!-- Unified Sidebar (always visible on desktop, drawer on mobile) -->
<div id="unified-sidebar"
     class="w-72 flex-shrink-0 bg-sidebar border-r border-border flex-col h-full overflow-hidden hidden lg:flex">
    <div id="unified-sidebar-content"
         hx-trigger="load once, memoUpdated from:body"
         hx-get="/unified-sidebar"
         hx-swap="innerHTML"
         class="flex flex-col h-full">
    </div>
</div>
```

### B. New Template: `UNIFIED_SIDEBAR_TEMPLATE`

Combines content from `SIDEBAR_TIMELINE_TEMPLATE` and `NOTES_PANEL_TEMPLATE`:

```html
<div class="flex flex-col h-full">
    <!-- Header -->
    <div class="px-4 py-3 border-b border-border flex-shrink-0">
        <h2 class="text-xs font-semibold text-muted-fg uppercase tracking-wider">Notes</h2>
    </div>

    <!-- Search -->
    <div class="px-3 pt-3 pb-2 flex-shrink-0">
        <input type="text" id="sidebar-search-input" name="q" placeholder="Search notes..."
            hx-get="/search"
            hx-target="#timeline"
            hx-swap="innerHTML"
            hx-trigger="keyup changed delay:400ms, search"
            hx-on::before-request="if (this.value === '') { event.detail.pathInfo.requestPath = '/memos-feed' }"
            hx-on::after-request="htmx.trigger('body', 'searchUpdated')"
            oninput="debouncedFilterSidebar(this.value)"
            class="w-full px-3 py-1.5 bg-card border border-border rounded-lg text-sm ..." />
    </div>

    <!-- Note List (primary content, scrollable) -->
    <div class="flex-1 overflow-y-auto p-2 space-y-0.5" id="notes-list-container">
        {% for note in notes %}
        <div data-note-id="{{ note.id }}" data-title="{{ note.title|e }}"
             data-search="{{ note.search_text|e }}"
             onclick="openNote({{ note.id }})"
             class="p-2.5 rounded-lg hover:bg-muted cursor-pointer transition-colors
                    flex gap-3 items-start justify-between">
            <div class="flex-1 min-w-0">
                <p class="note-title text-sm font-medium text-foreground truncate
                          flex items-center gap-1.5">
                    {{ note.title }}
                    <!-- visibility icon -->
                </p>
                <p class="text-xs text-muted-fg mt-0.5">{{ note.created_at }}</p>
                <!-- tags (compact) -->
            </div>
            <!-- thumbnail -->
        </div>
        {% endfor %}
        <!-- infinite scroll sentinel -->
    </div>

    <!-- Calendar (collapsible) -->
    <div class="border-t border-border flex-shrink-0">
        <button onclick="toggleSection('calendar-section')"
                class="w-full flex items-center justify-between px-3 py-2 text-xs
                       font-semibold text-muted-fg uppercase tracking-wider
                       hover:bg-muted/50 transition-colors">
            <span>Calendar</span>
            <div class="flex items-center gap-2">
                <span class="text-muted-fg font-normal normal-case">{{ month_label }}</span>
                <svg class="w-3 h-3 section-chevron transition-transform" ...>▾</svg>
            </div>
        </button>
        <div id="calendar-section" class="hidden px-3 pb-2"
             hx-trigger="searchUpdated from:body"
             hx-get="/calendar"
             hx-target="this"
             hx-swap="innerHTML"
             hx-include="#sidebar-search-input">
            <!-- calendar grid (same as current) -->
        </div>
    </div>

    <!-- Tags (collapsible) -->
    <div class="border-t border-border flex-shrink-0">
        <button onclick="toggleSection('tags-section')" ...>
            <span>Tags</span>
            <span>{{ tags|length }}</span>
        </button>
        <div id="tags-section" class="hidden px-3 pb-2">
            <div class="flex flex-wrap gap-1.5">
                {% for tag in tags %}
                <button hx-get="/search?tag={{ tag.name }}" ...>
                    #{{ tag.name }} <span>{{ tag.count }}</span>
                </button>
                {% endfor %}
            </div>
        </div>
    </div>
</div>
```

### C. Modify Icon Bar — Remove "Notes" Icon

**Before (3 nav items):**
```
Timeline | Notes | Resources
```

**After (2 nav items):**
```
Notes | Resources
```

Change the first icon from "Timeline" to "Notes" and update its `href` to `/app`. Remove the second icon (`icon-notes`) entirely.

```html
<a id="icon-notes" href="/app"
   class="p-2.5 rounded-xl {% if active_panel == 'notes' %}bg-accent-100 ...{% endif %}"
   title="Notes">
    <!-- Use the existing notes icon SVG (document with +) -->
</a>
<!-- Resources icon stays the same -->
```

### D. Modify Main Content — Support Feed ↔ Focused Toggle

The main content area needs to support both states via a simple JS toggle:

```html
<div id="main-content" class="flex-1 flex flex-col h-full overflow-hidden min-w-0">

    <!-- "Back" bar (hidden when in feed state) -->
    <div id="note-back-bar" class="hidden px-3 sm:px-4 lg:px-6 pt-3 flex-shrink-0">
        <button onclick="backToFeed()"
            class="flex items-center gap-1.5 text-sm text-muted-fg
                   hover:text-foreground transition-colors">
            <svg class="w-4 h-4" ...>← arrow</svg>
            Back to all notes
        </button>
    </div>

    <!-- Editor (hidden when focused on a note) -->
    <div id="editor-section">
        <!-- existing editor form, unchanged -->
    </div>

    <!-- Timeline (shows feed or focused note) -->
    <div id="timeline-view" class="flex-1 flex flex-col overflow-hidden">
        <div class="flex-1 overflow-y-auto px-3 sm:px-4 lg:px-6 py-3 lg:py-5 pb-20 lg:pb-5">
            <div class="max-w-2xl mx-auto">
                <div id="timeline" class="space-y-1">
                    <!-- memo cards (feed) or single focused note -->
                </div>
            </div>
        </div>
    </div>
</div>
```

### E. Modify Mobile Sidebar Drawer

The mobile drawer's content should load the unified sidebar instead of panel-specific content:

```html
<div id="mobile-sidebar-drawer" class="fixed top-0 left-0 z-50 h-full w-[85vw] max-w-[320px] ...">
    <div class="flex flex-col h-full">
        <div class="px-4 py-3 border-b border-border flex-shrink-0 flex items-center justify-between">
            <h2 class="text-xs font-semibold text-muted-fg uppercase tracking-wider">Notes</h2>
            <button onclick="closeMobileSidebar()" ...>✕</button>
        </div>
        <div id="mobile-sidebar-content" class="flex-1 overflow-y-auto"
             hx-trigger="load once, memoUpdated from:body"
             hx-get="/unified-sidebar"
             hx-swap="innerHTML">
        </div>
    </div>
</div>
```

### F. Modify Bottom Navigation (Mobile)

Remove the Notes tab, rename Timeline to Notes:

```html
<nav class="fixed bottom-0 left-0 right-0 z-30 ... lg:hidden safe-area-bottom">
    <a href="/app" class="flex flex-col items-center gap-0.5 p-1.5 ...">
        <!-- Notes icon -->
        <span class="text-[10px]">Notes</span>
    </a>
    <a href="/app/resources" class="flex flex-col items-center gap-0.5 p-1.5 ...">
        <!-- Resources icon -->
        <span class="text-[10px]">Resources</span>
    </a>
    <button onclick="openSettings()" class="flex flex-col items-center gap-0.5 p-1.5 ...">
        <!-- Settings icon -->
        <span class="text-[10px]">Settings</span>
    </button>
</nav>
```

### G. Keep Resources Panel Unchanged

The resources panel sidebar remains as-is, shown when `active_panel == 'resources'`.

---

## 10. Implementation Plan — Rust Routes

### Changes to `main.rs`

#### A. New HTMX endpoint: `/unified-sidebar`

Returns the unified sidebar HTML fragment. Combines data from `get_sidebar_timeline` and `get_notes_panel`:

```rust
async fn get_unified_sidebar(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    Query(params): Query<HashMap<String, String>>,
) -> impl IntoResponse {
    let user_id = match get_session_user_id(&headers) {
        Some(id) => id,
        None => return StatusCode::UNAUTHORIZED.into_response(),
    };
    let offset: i64 = params.get("offset").and_then(|v| v.parse().ok()).unwrap_or(0);
    let limit: i64 = 30;

    // Get note list (from existing notes panel logic)
    let memos = state.db.get_memos_paginated(user_id, limit + 1, offset).unwrap_or_default();
    let has_more = memos.len() as i64 > limit;
    let page_memos: Vec<_> = memos.into_iter().take(limit as usize).collect();
    let notes = /* ... map to note items, same as get_notes_panel ... */;

    // Get calendar data (from existing sidebar_timeline logic)
    let calendar_weeks = /* ... existing calendar logic ... */;
    let month_label = /* ... existing month label logic ... */;

    // Get tags (from existing sidebar_timeline logic)
    let tags = state.db.get_all_tags(user_id).unwrap_or_default();

    state.templates.render("unified_sidebar", &json!({
        "notes": notes,
        "calendar_weeks": calendar_weeks,
        "month_label": month_label,
        "tags": tags,
        "offset": offset,
        "next_offset": if has_more { Some(offset + limit) } else { None },
        "partial": offset > 0,
    })).into_response()
}
```

#### B. Modify `/app/notes` Route

Change `get_app_notes` to redirect to `/app` (or `/app/timeline`):

```rust
async fn get_app_notes(headers: HeaderMap, State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let user_id = match get_session_user_id(&headers) {
        Some(id) => id,
        None => return Redirect::to("/login").into_response(),
    };
    // Redirect to unified view
    Redirect::to("/app/timeline").into_response()
}
```

#### C. Modify `render_app_page`

Change `active_panel` to always be `"notes"` for the unified view:

```rust
async fn render_app_page(
    user_id: i64,
    state: &Arc<AppState>,
    active_panel: &str,
    selected_note_id: Option<i64>,
) -> Response<Body> {
    // ... existing logic, but active_panel is now always "notes" for the main view
}
```

#### D. Register the new route

```rust
.route("/unified-sidebar", get(get_unified_sidebar))
```

#### E. Keep All Existing Endpoints

All of these remain fully functional and unchanged:

- `/search` — search memos
- `/memos-feed` — infinite scroll feed
- `/memos/{id}/fragment` — single memo fragment
- `/memos/{id}/edit` — memo edit form
- `/calendar` — calendar widget
- `/notes-panel` — can be kept for backward compat, but not actively used
- `/sidebar-timeline` — can be kept for backward compat, but not actively used

---

## 11. Implementation Plan — JavaScript

### A. Modify `openNote()` Function

Currently, `openNote()` only works when the notes panel is visible. Update it to work universally:

```javascript
function openNote(id) {
    // Update sidebar highlight
    var timeline = document.getElementById('timeline');
    if (timeline) timeline.setAttribute('data-active-note-id', String(id));
    highlightActiveNote();

    // Hide editor, show back bar
    var editorSection = document.getElementById('editor-section');
    var backBar = document.getElementById('note-back-bar');
    if (editorSection) editorSection.classList.add('hidden');
    if (backBar) backBar.classList.remove('hidden');

    // Load the focused note into the timeline area
    htmx.ajax('GET', '/memos/' + id + '/fragment', {
        target: '#timeline',
        swap: 'innerHTML'
    });

    // Close mobile drawer if open
    closeMobileSidebar();
}
```

### B. New `backToFeed()` Function

```javascript
function backToFeed() {
    // Show editor, hide back bar
    var editorSection = document.getElementById('editor-section');
    var backBar = document.getElementById('note-back-bar');
    if (editorSection) editorSection.classList.remove('hidden');
    if (backBar) backBar.classList.add('hidden');

    // Clear active note highlight
    var timeline = document.getElementById('timeline');
    if (timeline) timeline.removeAttribute('data-active-note-id');
    highlightActiveNote();

    // Reload the timeline feed
    htmx.ajax('GET', '/memos-feed', {
        target: '#timeline',
        swap: 'innerHTML'
    });
}
```

### C. New `toggleSection()` Function (for collapsible calendar/tags)

```javascript
function toggleSection(sectionId) {
    var section = document.getElementById(sectionId);
    if (!section) return;
    var isHidden = section.classList.contains('hidden');
    section.classList.toggle('hidden');

    // Rotate chevron
    var btn = section.previousElementSibling;
    var chevron = btn ? btn.querySelector('.section-chevron') : null;
    if (chevron) {
        chevron.classList.toggle('rotate-180', !isHidden);
    }

    // Persist state
    localStorage.setItem('sidebar-' + sectionId, isHidden ? 'open' : 'closed');
}

// Restore collapsed state on load
document.addEventListener('DOMContentLoaded', function() {
    ['calendar-section', 'tags-section'].forEach(function(id) {
        var saved = localStorage.getItem('sidebar-' + id);
        var el = document.getElementById(id);
        if (!el) return;
        if (saved === 'open') {
            el.classList.remove('hidden');
        }
    });
});
```

### D. Modify `highlightActiveNote()`

Update to search within `#unified-sidebar` and `#notes-list-container` instead of `#notes-panel`:

```javascript
function highlightActiveNote() {
    var timeline = document.getElementById('timeline');
    var activeId = timeline ? timeline.getAttribute('data-active-note-id') : null;

    // Clear all highlights
    document.querySelectorAll('#notes-list-container [data-note-id]').forEach(function(el) {
        el.classList.remove('bg-accent-50', 'dark:bg-accent-900/20');
        var title = el.querySelector('.note-title');
        if (title) title.classList.remove('text-accent-600', 'dark:text-accent-600', 'font-semibold');
    });

    if (!activeId) return;

    // Highlight active note
    var selected = document.querySelector('#notes-list-container [data-note-id="' + activeId + '"]');
    if (selected) {
        selected.classList.add('bg-accent-50', 'dark:bg-accent-900/20');
        var title = selected.querySelector('.note-title');
        if (title) title.classList.add('text-accent-600', 'dark:text-accent-600', 'font-semibold');
    }
}
```

### E. Auto-Return to Feed After Delete

Add a handler so that if a user deletes the focused note, the view returns to the feed:

```javascript
// In the existing deleteMemo function, after successful delete:
function deleteMemo(id) {
    if (!confirm('Delete this note?')) return;
    var btn = document.querySelector('#memo-' + id + ' button[onclick*="deleteMemo"]');
    if (btn) btn.disabled = true;
    htmx.ajax('DELETE', '/memos/' + id, { target: '#memo-' + id, swap: 'outerHTML' });

    // If this was the focused note, go back to feed
    var timeline = document.getElementById('timeline');
    var activeId = timeline ? timeline.getAttribute('data-active-note-id') : null;
    if (activeId && String(activeId) === String(id)) {
        setTimeout(function() { backToFeed(); }, 300);
    }
}
```

---

## 12. Features Preserved

Every single existing feature remains fully functional:

| Feature | Status | Notes |
|:---|:---|:---|
| Create new notes (Tiptap editor) | ✅ | Unchanged — editor is in main area |
| Edit notes inline | ✅ | Same `editMemo()` / `cancelEdit()` flow |
| Delete notes | ✅ | Same `deleteMemo()` flow + auto-return to feed |
| Share notes | ✅ | Same `shareNote()` flow |
| Visibility (public/protected/private) | ✅ | Unchanged |
| Password-protected notes | ✅ | Unchanged |
| Search notes | ✅ | Enhanced — single search filters both sidebar and feed |
| Calendar date filtering | ✅ | Moved to collapsible section, same HTMX behavior |
| Tag filtering | ✅ | Moved to collapsible section, same HTMX behavior |
| Browse notes by title | ✅ | Note list always visible in sidebar |
| Note thumbnails | ✅ | Preserved in sidebar note list |
| Infinite scroll (timeline) | ✅ | Unchanged |
| Infinite scroll (note list) | ✅ | Unchanged sentinel behavior |
| Dark/light mode | ✅ | Unchanged |
| Accent themes | ✅ | Unchanged |
| Resources panel | ✅ | Completely unchanged |
| Slash commands | ✅ | Unchanged |
| Emoji picker | ✅ | Unchanged |
| File/image upload | ✅ | Unchanged |
| Audio recording | ✅ | Unchanged |
| Note linking | ✅ | Unchanged |
| Drag & drop | ✅ | Unchanged |
| HTMX partial swaps | ✅ | All endpoints preserved |
| Shared note pages | ✅ | Unchanged (`/share/`) |
| Code syntax highlighting | ✅ | Unchanged |
| Mobile responsive layout | ✅ | Simplified (fewer nav items) |

---

## 13. UX Improvements Summary

| Aspect | Before | After |
|:---|:---|:---|
| **Navigation items** | 3 (Timeline, Notes, Resources) | 2 (Notes, Resources) |
| **Pages to learn** | 3 separate views | 1 unified view + resources |
| **Finding a note** | "Do I search in Timeline or Notes?" | Always search in the sidebar |
| **Context switching** | Full page reload between views | Instant in-page toggle |
| **Calendar access** | Only in Timeline view | Always available (collapsible) |
| **Tags access** | Only in Timeline view | Always available (collapsible) |
| **Note list access** | Only in Notes view | Always visible in sidebar |
| **Mobile nav items** | 4 bottom tabs | 3 bottom tabs |
| **Cognitive load** | "Which view am I in?" | One view, one mental model |
| **New user onboarding** | Must discover 2 views | Single obvious interface |

### Estimated Implementation Size

| File | Lines Added | Lines Modified | Lines Removed |
|:---|:---|:---|:---|
| `src/templates.rs` | ~80 (unified sidebar template) | ~40 (icon bar, main content, JS) | ~30 (duplicate panel logic) |
| `src/main.rs` | ~40 (unified-sidebar handler) | ~10 (route changes) | ~5 (notes redirect) |
| **Total** | **~120** | **~50** | **~35** |

**RAM impact:** Zero. Same number of HTMX endpoints, same template rendering pattern, one less page load type.
