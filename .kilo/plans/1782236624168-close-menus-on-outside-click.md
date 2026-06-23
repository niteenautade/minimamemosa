# Plan: Close Menus on Outside Click

## Problem
Clicking outside the emoji picker, plus icon menu, and privacy dropdown does not close them. No document-level click-outside handler exists for these elements.

## Root Cause
1. `closeAllDropdowns()` (templates.rs:1100) only handles `emoji-picker`, `plus-menu`, `link-memo-dropdown` — **missing** `vis-dropdown-menu`.
2. No `document.addEventListener('click', ...)` closes these menus when the user clicks outside.
3. The existing pattern for `image-resize-menu` (templates.rs:1280–1285) proves this approach works in this codebase.

## Changes (templates.rs)

### 1. Update `closeAllDropdowns()` (line ~1101)
Add the visibility dropdown IDs to the array:
```js
function closeAllDropdowns() {
    ['emoji-picker','plus-menu','link-memo-dropdown','vis-dropdown-menu'].forEach(function(id) {
        var el = document.getElementById(id);
        if (el) el.classList.add('hidden');
    });
}
```

### 2. Add document-level click-outside listener (after existing listeners, near line 1595)
```js
document.addEventListener('click', function(e) {
    var emoji = document.getElementById('emoji-picker');
    var plus  = document.getElementById('plus-menu');
    var link  = document.getElementById('link-memo-dropdown');
    var vis   = document.getElementById('vis-dropdown-menu');
    if (emoji && !emoji.classList.contains('hidden') && !emoji.contains(e.target)) emoji.classList.add('hidden');
    if (plus  && !plus.classList.contains('hidden')  && !plus.contains(e.target))  plus.classList.add('hidden');
    if (link  && !link.classList.contains('hidden')  && !link.contains(e.target))  link.classList.add('hidden');
    if (vis   && !vis.classList.contains('hidden')   && !vis.contains(e.target))   vis.classList.add('hidden');
});
```

### 3. Stop propagation on toggle buttons (defensive)
Add `onclick="event.stopPropagation()"` to the emoji picker toggle button, plus menu toggle button, and visibility dropdown toggle button so the document click handler doesn't fire when intentionally opening a menu.

## Edge Cases
- **Plus menu → Link Note sub-dropdown**: The `link-memo-dropdown` is a child of `#plus-menu`, so `plus-menu.contains(e.target)` naturally covers it. Clicking inside link-memo-dropdown won't close plus-menu.
- **Clicking a visibility option**: `selectVis()` calls `applyVis()` → `dd.querySelector('.vis-dropdown-menu').classList.add('hidden')`, which runs before the document handler fires, so no conflict.
- **htmx swaps**: Newly injected memo fragments include their own visibility dropdowns; the IDs are consistent so the document listener works on them automatically.

## Validation
1. Open emoji picker → click outside → picker closes.
2. Open plus menu → click outside → menu closes.
3. Open link-memo dropdown (via plus menu) → click outside → both close.
4. Open privacy dropdown → click outside → dropdown closes.
5. Select a visibility option → dropdown closes (existing behavior preserved).
6. Clicking inside any open dropdown/menu does not close it.
