# Image Handler — Editor UX Specification

This document defines the complete user experience for images and text coexistence inside the ScrapNotes editor (TipTap/ProseMirror). Every interaction described here should feel invisible and intuitive — the user should never have to think about "how" to place text around images.

---

## 1. Core Principles

1. **Images are block-level elements.** An image always occupies its own visual row, centered horizontally within the editor. It never shares a line with text.
2. **Text lives above or below images, never beside them.** The editor does not support text wrapping around images. This keeps the note-taking flow linear and predictable.
3. **Clicking near an image creates a text insertion point.** The user should always be able to start typing around an image with a single click — no keyboard shortcuts or special gestures required.
4. **No orphan images.** Every image should always be reachable by the cursor from both above and below. The editor must guarantee that a paragraph node exists (or can be created on demand) before and after every image.

---

## 2. Image Layout

| Property         | Value                                              |
| :--------------- | :------------------------------------------------- |
| Display          | `block`                                            |
| Horizontal align | Centered via `margin: auto`                        |
| Default width    | `75%` of editor width                              |
| Max width        | `100%` of editor width                             |
| Vertical spacing | `0.5rem` margin top and bottom                     |
| Border radius    | `0.5rem` (rounded corners)                         |
| Cursor on hover  | `default` (pointer over image), `text` over margins|

---

## 3. Cursor Behavior — Single Image

### 3.1 Click Zones

When an image is rendered, the editor area is divided into three horizontal zones on the image's row:

```
┌─────────────────────────────────────────────────┐
│  LEFT MARGIN  │      IMAGE (75%)      │ RIGHT   │
│   (zone A)    │      (zone B)         │ MARGIN  │
│               │                       │(zone C) │
└─────────────────────────────────────────────────┘
```

- **Zone A — Left of the image** (`clientX < image.getBoundingClientRect().left`):  
  Clicking here places a blinking text cursor at the **left edge of the image row**. When the user begins typing, a new paragraph is **inserted above the image**, and the typed text flows into that paragraph. The cursor transitions seamlessly from the image row into the new paragraph.

- **Zone C — Right of the image** (`clientX > image.getBoundingClientRect().right`):  
  Clicking here places a blinking text cursor at the **right edge of the image row**. When the user begins typing, a new paragraph is **inserted below the image**, and the typed text flows into that paragraph.

- **Zone B — On the image itself**:  
  A single click selects the image node (ProseMirror node selection). Double-click opens the image resize menu.

### 3.2 Visual Cursor Appearance

- The cursor must be a standard-height text caret (matching the editor's base `line-height`), **not** stretched to the full height of the image.
- When the cursor is placed at the left margin (Zone A), it should appear vertically aligned with the **bottom edge** of the image, flush to the left margin of the editor.
- When the cursor is placed at the right margin (Zone C), it should appear vertically aligned with the **bottom edge** of the image, flush to the right side of the image.

### 3.3 Typing Flow

```
BEFORE typing (cursor at left of image):

    |                                        ← cursor blinks here
    ┌──────────────────────────┐
    │         IMAGE            │
    └──────────────────────────┘

AFTER user starts typing:

    Hello, this is my caption above         ← new paragraph created above
    ┌──────────────────────────┐
    │         IMAGE            │
    └──────────────────────────┘
```

```
BEFORE typing (cursor at right of image):

    ┌──────────────────────────┐
    │         IMAGE            │
    └──────────────────────────┘
                                        |   ← cursor blinks here

AFTER user starts typing:

    ┌──────────────────────────┐
    │         IMAGE            │
    └──────────────────────────┘
    This text appears below the image       ← new paragraph created below
```

---

## 4. Cursor Behavior — Multiple Consecutive Images

When the user pastes or inserts multiple images in sequence, the editor must handle the gaps between them gracefully.

### 4.1 Layout

```
    ┌──────────────────────────┐
    │         IMAGE 1          │
    └──────────────────────────┘
                                            ← implicit gap (clickable)
    ┌──────────────────────────┐
    │         IMAGE 2          │
    └──────────────────────────┘
                                            ← implicit gap (clickable)
    ┌──────────────────────────┐
    │         IMAGE 3          │
    └──────────────────────────┘
```

### 4.2 Click Behavior Between Images

- Clicking the **gap area between two images** (the vertical space between them) should insert an empty paragraph between the two image nodes and place the cursor inside it.
- This lets the user add captions, annotations, or any text between photos without needing to use arrow keys.

### 4.3 Edge Cases

| Scenario | Expected Behavior |
| :--- | :--- |
| Image is the **first node** in the document | Clicking left margin inserts a paragraph **before** the image (at document position 0) |
| Image is the **last node** in the document | Clicking right margin inserts a paragraph **after** the image (at document end) |
| Two images are **directly adjacent** (no paragraph between them) | Clicking Zone C of Image 1 or Zone A of Image 2 inserts a paragraph **between** them |
| Image is **inside a blockquote or list item** | Same rules apply, scoped to the parent container |
| User **deletes all text** above an image | The empty paragraph is removed, but the image remains reachable via arrow keys from the top of the document |

### 4.4 Arrow Key Navigation

- **↓ (Down Arrow)** from a paragraph above an image: Selects the image (node selection).
- **↓ (Down Arrow)** from a selected image: Moves cursor to the paragraph below (or creates one if none exists).
- **↑ (Up Arrow)** from a paragraph below an image: Selects the image.
- **↑ (Up Arrow)** from a selected image: Moves cursor to the paragraph above (or creates one if none exists).
- **Enter** while an image is selected: Inserts an empty paragraph below the image and places the cursor there.
- **Backspace** while an image is selected: Deletes the image.

---

## 5. Image Insertion

When an image is inserted (via paste, drag-and-drop, or slash command `/image`):

1. The image node is inserted at the current cursor position.
2. An empty paragraph is automatically appended **after** the image node.
3. The cursor is placed inside this trailing paragraph so the user can immediately continue typing below.

If the image is inserted at the very beginning of the document and there is no content above:
- An empty paragraph should **also** be prepended above the image, ensuring the user can always navigate and type above the first image.

---

## 6. Image Resizing

- **Double-click** on an image opens a resize popover menu.
- Resize options: 25%, 50%, 75%, 100% width.
- After resizing, the image remains centered and block-level. The cursor zones (A, B, C) adjust to the new image width.

---

## 7. Implementation Notes

### TipTap Extension Configuration

```
Image.extend({
    // Block-level (default), NOT inline
    // No group override needed — uses default 'block' group
    addAttributes() → src, alt, title, style (default: 'width: 75%')
    renderHTML()    → plain <img> tag with style attribute
    parseHTML()     → parses <img[src]>, extracts width from query params
})
```

### ProseMirror `handleClick` Plugin

The click handler intercepts clicks on the editor and checks:

1. Did the click target an `<img>` element's **margin area** (Zone A or Zone C)?
2. If yes:
   - Determine the image's document position via `view.posAtDOM()`
   - Determine click side (left vs right)
   - Insert a `<p></p>` at the correct position (before image for left, after image for right)
   - Set text selection inside the new paragraph
   - Focus the editor
   - Return `true` to prevent default ProseMirror handling

### CSS Requirements

```css
.tiptap-editor .ProseMirror img {
    display: block;
    max-width: 100%;
    height: auto;
    border-radius: 0.5rem;
    margin: 0.5rem auto;
    cursor: default;
}
```

No `text-align: center` on `p:has(img)` is needed — the image centers itself via `margin: auto`.

---

## 8. Anti-Patterns to Avoid

| ❌ Don't | ✅ Do Instead |
| :--- | :--- |
| Make images `inline` or `inline-block` nodes | Keep images as `block` nodes |
| Use a wrapper `<span>` or `<div>` around `<img>` in renderHTML | Return a plain `<img>` tag |
| Set `cursor: text` on the image itself | Use `cursor: default` on images; the click handler manages cursor placement |
| Stretch the cursor caret to match image height | Keep the caret at standard text line-height |
| Allow text on the same line as an image | Always push text to a separate paragraph above or below |
| Force `text-align: center` on image paragraphs | Center images with `margin: auto` on the `<img>` itself |

---

## 9. Summary

The image experience in ScrapNotes should feel like writing in a notebook where photos are taped in: you write above them, you write below them, but you never write *over* or *beside* them. Every click near an image should result in a visible, correctly-sized cursor that immediately lets you type. The editor handles the structural bookkeeping (inserting paragraphs, managing positions) so the user never has to.
