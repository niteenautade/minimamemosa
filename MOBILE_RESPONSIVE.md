# MinimaMemosa — Mobile Responsive Design Plan

> **Goal:** Make MinimaMemosa fully usable on mobile devices (320px–768px) and tablets (768px–1024px) without breaking any existing desktop features.  
> **Approach:** Progressive enhancement via Tailwind responsive utilities + minimal custom CSS.  
> **Constraint:** Zero additional JS runtime libraries. No RAM budget increase. All changes are CSS/HTML template-only.

---

## Table of Contents

1. [Breakpoint Strategy](#1-breakpoint-strategy)
2. [Layout Architecture Changes](#2-layout-architecture-changes)
3. [Header — Mobile Adaptation](#3-header--mobile-adaptation)
4. [Icon Bar — Bottom Navigation](#4-icon-bar--bottom-navigation)
5. [Sidebar Panels — Slide-Over Drawer](#5-sidebar-panels--slide-over-drawer)
6. [Main Content — Timeline & Editor](#6-main-content--timeline--editor)
7. [Memo Cards](#7-memo-cards)
8. [Editor Toolbar & Slash Commands](#8-editor-toolbar--slash-commands)
9. [Modals — Full-Width on Mobile](#9-modals--full-width-on-mobile)
10. [Login & Register Pages](#10-login--register-pages)
11. [Shared Note View](#11-shared-note-view)
12. [Touch Interactions](#12-touch-interactions)
13. [Implementation Details — File-by-File](#13-implementation-details--file-by-file)
14. [Testing Checklist](#14-testing-checklist)
15. [Features Preserved](#15-features-preserved)

---

## 1. Breakpoint Strategy

Use Tailwind's default breakpoints. All changes are mobile-first overrides.

| Breakpoint | Width | Target |
|:---|:---|:---|
| Default (no prefix) | 0–639px | Mobile phones (portrait) |
| `sm:` | 640–767px | Large phones (landscape) |
| `md:` | 768–1023px | Tablets |
| `lg:` | 1024px+ | Desktop (current layout — **no changes**) |

**Critical rule:** Every `lg:` class preserves the current desktop behavior exactly. Mobile classes only add overrides for smaller screens.

---

## 2. Layout Architecture Changes

### Current Desktop Layout (preserved at `lg:`)

```
┌──────────────────────────────────────────────┐
│ Header (full width, fixed height)            │
├──────┬──────────┬────────────────────────────┤
│ Icon │ Sidebar  │ Main Content               │
│ Bar  │ (w-72)   │ (flex-1)                   │
│(w-14)│          │                            │
│      │          │                            │
└──────┴──────────┴────────────────────────────┘
```

### Mobile Layout (< 1024px)

```
┌──────────────────────────────┐
│ Header (compact, hamburger)  │
├──────────────────────────────┤
│                              │
│     Main Content             │
│     (full width, scrollable) │
│                              │
├──────────────────────────────┤
│ Bottom Nav Bar (Icon Bar)    │
└──────────────────────────────┘

   ┌── Sidebar Drawer ──┐
   │  (slide-in overlay) │  ← opened via hamburger/swipe
   └─────────────────────┘
```

### Key Structural Changes

The outer flex container (`flex flex-col h-screen overflow-hidden`) and inner flex row (`flex flex-1 overflow-hidden`) remain, but:

- **Icon Bar (`w-14`):** Hidden on mobile (`hidden lg:flex`). Replaced by a fixed bottom navigation bar (`fixed bottom-0 lg:hidden`).
- **Sidebar Panels (`w-72`):** Hidden by default on mobile (`hidden lg:flex`). Toggled visible as a slide-over drawer via a hamburger button added to the mobile header.
- **Main Content:** Becomes full-width on mobile (`w-full lg:flex-1`).

---

## 3. Header — Mobile Adaptation

### Current Header (line ~587–608 in `templates.rs`)

The header is a horizontal flex bar with app name, theme toggle, avatar, and logout.

### Mobile Changes

| Element | Desktop (lg:) | Mobile (< lg) |
|:---|:---|:---|
| App name | `text-sm font-semibold` | Stays the same |
| Hamburger button | Hidden (`hidden lg:hidden`) | Visible (`lg:hidden`), toggles sidebar drawer |
| Theme toggle | Visible | Stays visible |
| Avatar + username | Visible | Avatar only, hide username text (`hidden sm:inline`) |
| Logout icon | Visible | Stays visible (icon-only is fine) |

### New Element: Hamburger Button

Add a hamburger menu button before the app name on mobile:

```html
<button onclick="toggleMobileSidebar()"
    class="p-1.5 rounded-lg hover:bg-muted text-muted-fg transition-colors lg:hidden"
    title="Menu">
    <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h16"/>
    </svg>
</button>
```

---

## 4. Icon Bar — Bottom Navigation

### Current Icon Bar (line ~612–651)

A vertical `w-14` strip with icons for Timeline, Notes, Resources, and Settings.

### Mobile Replacement

Hide the vertical icon bar on mobile and show a fixed bottom navigation bar:

```html
<!-- Bottom Nav (mobile only) -->
<nav class="fixed bottom-0 left-0 right-0 z-30 bg-card border-t border-border 
            flex items-center justify-around py-2 px-4 lg:hidden safe-area-bottom">
    <!-- Same icon links as the vertical bar -->
    <a href="/app/timeline" class="flex flex-col items-center gap-0.5 p-1.5 ...">
        <!-- Timeline SVG -->
        <span class="text-[10px]">Timeline</span>
    </a>
    <a href="/app/notes" class="flex flex-col items-center gap-0.5 p-1.5 ...">
        <!-- Notes SVG -->
        <span class="text-[10px]">Notes</span>
    </a>
    <a href="/app/resources" class="flex flex-col items-center gap-0.5 p-1.5 ...">
        <!-- Resources SVG -->
        <span class="text-[10px]">Resources</span>
    </a>
    <button onclick="openSettings()" class="flex flex-col items-center gap-0.5 p-1.5 ...">
        <!-- Settings SVG -->
        <span class="text-[10px]">Settings</span>
    </button>
</nav>
```

### CSS Addition

```css
/* Safe area for iPhone notch/gesture bar */
.safe-area-bottom {
    padding-bottom: max(0.5rem, env(safe-area-inset-bottom));
}
```

The original vertical icon bar gets `hidden lg:flex` to hide on mobile.

---

## 5. Sidebar Panels — Slide-Over Drawer

### Current Sidebar (lines ~655–680)

Three sidebar panels (`sidebar-panel`, `notes-panel`, `resources-panel`), each `w-72`, shown/hidden via Tailwind classes.

### Mobile Strategy

On mobile, all sidebar panels are wrapped in a drawer overlay:

```html
<!-- Mobile Sidebar Drawer -->
<div id="mobile-sidebar-overlay" 
     class="fixed inset-0 z-40 bg-black/50 backdrop-blur-sm hidden lg:hidden"
     onclick="closeMobileSidebar()">
</div>
<div id="mobile-sidebar-drawer" 
     class="fixed top-0 left-0 z-50 h-full w-[85vw] max-w-[320px] bg-sidebar border-r border-border 
            transform -translate-x-full transition-transform duration-300 ease-in-out lg:hidden">
    <!-- Dynamic content loaded based on active panel -->
</div>
```

### JavaScript (added to `<script>` block in base template)

```javascript
function toggleMobileSidebar() {
    var overlay = document.getElementById('mobile-sidebar-overlay');
    var drawer  = document.getElementById('mobile-sidebar-drawer');
    if (!overlay || !drawer) return;
    var isOpen = !overlay.classList.contains('hidden');
    if (isOpen) {
        closeMobileSidebar();
    } else {
        overlay.classList.remove('hidden');
        drawer.classList.remove('-translate-x-full');
        drawer.classList.add('translate-x-0');
    }
}

function closeMobileSidebar() {
    var overlay = document.getElementById('mobile-sidebar-overlay');
    var drawer  = document.getElementById('mobile-sidebar-drawer');
    if (overlay) overlay.classList.add('hidden');
    if (drawer) {
        drawer.classList.remove('translate-x-0');
        drawer.classList.add('-translate-x-full');
    }
}
```

### Desktop Behavior (unchanged)

At `lg:` breakpoint, the existing sidebar panels remain inline (`lg:flex`, `lg:w-72`), and the mobile drawer elements are hidden (`lg:hidden`).

### Sidebar Content on Mobile

When a bottom nav icon is tapped:
1. The main content area updates (existing HTMX behavior preserved).
2. On mobile, tapping the hamburger opens the drawer with the sidebar content for the active panel.

Alternatively, the sidebar content (search, calendar, tags, note list, resource list) can be rendered **inside** the mobile drawer using the same HTMX endpoints — no new server routes needed.

---

## 6. Main Content — Timeline & Editor

### Timeline Scrollable Area (line ~686)

```html
<!-- Current -->
<div class="flex-1 overflow-y-auto px-6 py-5">
    <div class="max-w-2xl mx-auto">

<!-- Mobile-adapted -->
<div class="flex-1 overflow-y-auto px-3 sm:px-4 lg:px-6 py-3 lg:py-5 pb-20 lg:pb-5">
    <div class="max-w-2xl mx-auto">
```

Key adjustments:
- **Padding:** Reduced horizontal padding on mobile (`px-3` → `lg:px-6`).
- **Bottom padding:** `pb-20` on mobile to account for the bottom navigation bar height.

### Editor Container (line ~689–791)

The Notion-style editor block needs responsive padding:

```html
<!-- Current -->
<div class="px-8 pt-6 pb-2 relative">

<!-- Mobile-adapted -->
<div class="px-3 sm:px-4 lg:px-8 pt-4 lg:pt-6 pb-2 relative">
```

### Editor Toolbar (line ~718–789)

The bottom toolbar with emoji, plus menu, visibility dropdown, and Save button:

```html
<!-- Current -->
<div class="flex items-center justify-between px-8 py-3 border-t border-border ...">

<!-- Mobile-adapted -->
<div class="flex items-center justify-between px-3 sm:px-4 lg:px-8 py-2 lg:py-3 border-t border-border ...">
```

The toolbar items should wrap on very small screens:

```html
<div class="flex items-center flex-wrap gap-1 lg:gap-1">
```

### Editor Min-Height

Reduce editor minimum height on mobile for faster composing:

```html
<!-- Current -->
min-h-[10rem]

<!-- Mobile-adapted -->
min-h-[6rem] lg:min-h-[10rem]
```

### Keyboard Shortcut Hints

Hide `Ctrl+Enter` and `/` hints on mobile since they're irrelevant for touch:

```html
<span class="text-xs text-muted-fg hidden lg:inline">
    Press <kbd>...</kbd> for commands
</span>
```

---

## 7. Memo Cards

### Current Memo Fragment (line ~1942–1989)

Cards are already `max-w-2xl` and work well at smaller widths. Minor tweaks:

| Element | Change |
|:---|:---|
| Card padding | `p-3 lg:p-4` (slightly tighter on mobile) |
| Action buttons (edit/delete/share) | Always visible on mobile (no hover-to-reveal since no hover on touch): `opacity-100 lg:opacity-0 lg:group-hover/memo:opacity-100` |
| Memo content images | Already `max-width: 100%` ✓ |
| Tags row | Already `flex-wrap` ✓ |

### Action Buttons — Touch Visibility

The current `opacity-0 group-hover/memo:opacity-100` hides actions until hover. This doesn't work on touch screens.

**Fix:** Always show on mobile, hover-reveal on desktop:

```html
<!-- Current -->
<div class="ml-auto flex items-center gap-1 opacity-0 group-hover/memo:opacity-100 transition-opacity">

<!-- Mobile-adapted -->
<div class="ml-auto flex items-center gap-1 opacity-100 lg:opacity-0 lg:group-hover/memo:opacity-100 transition-opacity">
```

---

## 8. Editor Toolbar & Slash Commands

### Slash Menu Positioning

The slash command dropdown (`#slash-menu`) uses `position: fixed` with calculated coordinates. This already handles viewport bounds (`Math.max`, `Math.min` clamping), so it works on mobile. However:

- Ensure the menu doesn't overflow the viewport width:
  ```css
  #slash-menu {
      max-width: calc(100vw - 16px);
  }
  ```

### Emoji Picker

The emoji picker (`w-[280px]`) may overflow on very narrow screens:

```html
<!-- Current -->
class="... w-[280px] max-h-[200px] ..."

<!-- Mobile-adapted -->
class="... w-[280px] max-w-[calc(100vw-2rem)] max-h-[200px] ..."
```

### Plus Menu & Link Memo Dropdown

Similar max-width constraints:

```html
class="... min-w-[180px] max-w-[calc(100vw-2rem)] ..."
```

### Image Resize Menu

The context menu for image resizing (`#image-resize-menu`) should also be viewport-clamped. The existing JS already does `Math.max(4, Math.min(...))` — this is sufficient.

---

## 9. Modals — Full-Width on Mobile

### Current Modal Structure

All modals (share, visibility password, settings) use:
```html
<div class="... w-full max-w-sm mx-4">
```

This is already responsive! The `mx-4` provides edge margins and `max-w-sm` caps width. **No changes needed** for modal sizing.

### Settings Modal — Theme Swatches

The theme swatch colors (`w-4 h-4` grid) are small enough to work on mobile. No changes needed.

### Image Modal (Fullscreen Preview)

Already uses `max-width: 95vw; max-height: 95vh` — works perfectly on mobile. ✓

---

## 10. Login & Register Pages

### Current Structure (lines ~511–581)

```html
<div class="flex items-center justify-center min-h-screen">
    <div class="w-full max-w-sm mx-4">
```

This is already fully responsive. The form is centered, max-width capped, and has side margins. **No changes needed.**

### Captcha Image

The captcha in the register form (`h-16 object-contain`) scales properly. ✓

---

## 11. Shared Note View

### Current Structure (line ~1766–1802)

```html
<div class="flex items-center justify-center min-h-screen py-10">
    <div class="w-full max-w-2xl mx-4 bg-card rounded-xl ...">
```

Already responsive. The `mx-4` provides margins and `max-w-2xl` caps width. ✓

Minor improvement — reduce vertical padding on mobile:

```html
<div class="... py-4 lg:py-10">
    <div class="w-full max-w-2xl mx-3 lg:mx-4 ... p-4 lg:p-6">
```

---

## 12. Touch Interactions

### Hover States

All `hover:` effects work fine on touch devices as tap-and-hold. No changes needed, but the memo action buttons visibility fix (Section 7) is critical.

### Scroll Behavior

Add smooth scrolling and momentum scrolling for iOS:

```css
@media (max-width: 1023px) {
    .overflow-y-auto {
        -webkit-overflow-scrolling: touch;
    }
}
```

### Viewport Meta Tag

Already present: `<meta name="viewport" content="width=device-width, initial-scale=1.0">` ✓

### iOS Safari — 100vh Fix

The `h-screen` (100vh) can be problematic on iOS Safari due to the address bar. Fix:

```css
@supports (height: 100dvh) {
    .h-screen {
        height: 100dvh;
    }
}
```

### Prevent Double-Tap Zoom on Action Buttons

```css
button, a {
    touch-action: manipulation;
}
```

### Input Focus Zoom Prevention (iOS)

iOS zooms into inputs with `font-size < 16px`. Ensure all inputs are at least 16px:

```css
@media (max-width: 1023px) {
    input, textarea, select, .tiptap-editor .ProseMirror {
        font-size: 16px !important;
    }
}
```

---

## 13. Implementation Details — File-by-File

### File: `src/templates.rs`

This is the only file that needs modification. All changes are in the HTML template string constants and the `<style>` block.

#### A. `BASE_TEMPLATE` — Style Block Additions (line ~131–254)

Add new CSS rules at the end of the `<style>` block (before `</style>`):

```css
/* ── Mobile Responsive ── */

/* iOS 100vh fix */
@supports (height: 100dvh) {
    .h-screen, .min-h-screen { height: 100dvh; min-height: 100dvh; }
}

/* Prevent iOS input zoom */
@media (max-width: 1023px) {
    input, textarea, select, .tiptap-editor .ProseMirror {
        font-size: 16px !important;
    }
}

/* Safe area padding for bottom nav */
.safe-area-bottom {
    padding-bottom: max(0.5rem, env(safe-area-inset-bottom));
}

/* Touch optimization */
button, a { touch-action: manipulation; }

/* Smooth mobile scrolling */
@media (max-width: 1023px) {
    .overflow-y-auto { -webkit-overflow-scrolling: touch; }
}

/* Slash menu mobile constraint */
#slash-menu { max-width: calc(100vw - 16px); }

/* Mobile sidebar drawer transitions */
#mobile-sidebar-drawer {
    transition: transform 300ms cubic-bezier(0.4, 0, 0.2, 1);
}
```

#### B. `BASE_TEMPLATE` — Add Mobile Sidebar Drawer (before `</body>`)

Insert the drawer overlay + drawer container HTML.

#### C. `BASE_TEMPLATE` — Add JavaScript Functions

Add `toggleMobileSidebar()` and `closeMobileSidebar()` to the `<script>` block.

#### D. `TIMELINE_TEMPLATE` — Header (line ~587–608)

1. Add hamburger button (mobile only, `lg:hidden`).
2. Hide username text on small screens (`hidden sm:inline`).

#### E. `TIMELINE_TEMPLATE` — Icon Bar (line ~612–651)

Add `hidden lg:flex` to hide on mobile.

#### F. `TIMELINE_TEMPLATE` — Bottom Navigation

Insert bottom nav HTML after the main layout div, before `<script>` (mobile only, `lg:hidden`).

#### G. `TIMELINE_TEMPLATE` — Sidebar Panels (lines ~655–680)

No class changes needed on desktop — they already have conditional `hidden` logic. On mobile, the sidebar content is rendered inside the mobile drawer instead.

#### H. `TIMELINE_TEMPLATE` — Main Content (line ~683–686)

Adjust padding classes:
- `px-6` → `px-3 sm:px-4 lg:px-6`
- `py-5` → `py-3 lg:py-5`
- Add `pb-20 lg:pb-5` for bottom nav clearance.

#### I. `TIMELINE_TEMPLATE` — Editor Padding (line ~700, ~718)

Reduce `px-8` → `px-3 sm:px-4 lg:px-8` on editor content and toolbar.

#### J. `TIMELINE_TEMPLATE` — Keyboard Hints (line ~712, ~783)

Wrap in `hidden lg:block` / `hidden lg:inline`.

#### K. `MEMO_FRAGMENT` — Action Buttons (line ~1958)

Change `opacity-0 group-hover/memo:opacity-100` to `opacity-100 lg:opacity-0 lg:group-hover/memo:opacity-100`.

#### L. `MEMO_FRAGMENT` — Card Padding (line ~1942)

Change `p-4` to `p-3 lg:p-4`.

#### M. `MEMO_EDIT_FORM` — Editor Padding (line ~2000)

Change `px-4` to `px-3 lg:px-4`.

#### N. `RESOURCES_PANEL_TEMPLATE`

No changes needed — it's rendered inside the sidebar which becomes a drawer on mobile.

#### O. `SIDEBAR_TIMELINE_TEMPLATE`

No changes needed — it's rendered inside the sidebar which becomes a drawer on mobile.

---

## 14. Testing Checklist

### Devices to Test

- [ ] iPhone SE (375px) — smallest common phone
- [ ] iPhone 14 Pro (393px) — standard modern phone
- [ ] iPhone 14 Pro Max (430px) — large phone
- [ ] iPad Mini (768px) — small tablet
- [ ] iPad Air (820px) — standard tablet
- [ ] Generic Android (360px, 412px)

### Feature Checklist

| Feature | Test |
|:---|:---|
| Login page | Renders centered, inputs usable, no zoom on focus |
| Register page | Captcha visible, form usable |
| Timeline view | Full-width, scrollable, bottom nav visible |
| Create new memo | Editor opens, slash commands work, save button reachable |
| Edit memo | Inline editor opens, toolbar usable |
| Delete memo | Confirmation dialog appears, memo removed |
| Share note | Toast shows link copied |
| Dark/light toggle | Works from header button |
| Sidebar — Timeline | Opens via hamburger, search works, calendar usable |
| Sidebar — Notes | Opens via hamburger, note list scrollable, tap opens note |
| Sidebar — Resources | Opens via hamburger, upload button works, resources listed |
| Bottom nav | All 4 icons work, active state highlights |
| Settings modal | Opens, theme swatches selectable, save works |
| Visibility dropdown | Opens, options selectable |
| Password modal | Opens, inputs focusable, confirm works |
| Emoji picker | Opens, scrollable, emojis insertable |
| Plus menu | Opens, image/file upload works |
| Image modal (fullscreen) | Opens on tap, close button works |
| Image resize menu | Opens on tap (editor), options work |
| Shared note page | Renders properly at mobile width |
| Protected note page | Password input works |
| Infinite scroll (sentinels) | Loads more memos on scroll |
| HTMX fragment swaps | All swaps work correctly |
| Drag & drop (editor) | Touch equivalent: use Upload button |

---

## 15. Features Preserved

Every single existing feature remains fully functional:

| Feature | Status |
|:---|:---|
| Server-side rendering (SSR via HTMX) | ✅ Unchanged |
| HTMX partial swaps | ✅ Unchanged — all `hx-*` attributes preserved |
| Tiptap rich text editor | ✅ Unchanged — Tiptap handles mobile input natively |
| Markdown rendering | ✅ Unchanged |
| Slash commands | ✅ Unchanged — menu position clamped to viewport |
| Image upload/paste/drop | ✅ Unchanged — touch users use Upload button |
| Audio recording | ✅ Unchanged — `getUserMedia` works on mobile browsers |
| File attachments | ✅ Unchanged |
| Resource management | ✅ Unchanged |
| Note linking | ✅ Unchanged |
| Search (sidebar + timeline) | ✅ Unchanged |
| Calendar date filtering | ✅ Unchanged |
| Tag filtering | ✅ Unchanged |
| Light/dark mode toggle | ✅ Unchanged — `localStorage` based |
| Accent theme selection | ✅ Unchanged |
| Visibility (public/protected/private) | ✅ Unchanged |
| Share links | ✅ Unchanged |
| Password-protected notes | ✅ Unchanged |
| Infinite scroll | ✅ Unchanged — sentinels trigger on mobile scroll |
| Code syntax highlighting | ✅ Unchanged |
| Session cookies | ✅ Unchanged |

### RAM Impact

**Zero.** All changes are CSS classes and HTML template modifications. No new server-side routes, no new database queries, no new JavaScript libraries. The Tailwind CDN already handles responsive utilities.

---

## Summary of Changes

| Area | Type | Scope |
|:---|:---|:---|
| CSS additions (~30 lines) | New responsive rules | `<style>` block in `BASE_TEMPLATE` |
| Hamburger button | New HTML element | `TIMELINE_TEMPLATE` header |
| Bottom navigation bar | New HTML element | `TIMELINE_TEMPLATE` (mobile only) |
| Mobile sidebar drawer | New HTML elements | `BASE_TEMPLATE` body |
| `toggleMobileSidebar()` / `closeMobileSidebar()` | New JS functions (~20 lines) | `BASE_TEMPLATE` script |
| Responsive class adjustments | Class modifications | ~15 elements across templates |
| Action button visibility | Class modification | `MEMO_FRAGMENT` |
| Keyboard hints | Hide on mobile | `TIMELINE_TEMPLATE` |

**Total estimated diff:** ~120 lines added, ~15 lines modified, 0 lines removed.  
**Files touched:** `src/templates.rs` (only file).
