# Fix Edit Note Functionality

## Problem
The edit note functionality is not working because the `get_memo_edit_form` handler incorrectly returns the full `memo_fragment` template instead of just the edit form (`MEMO_EDIT_FORM`). This causes nested memo fragments when the edit form is loaded into the edit div.

## Root Cause
In `src/main.rs`, the `get_memo_edit_form` function (lines 1074-1109) renders `memo_fragment` template when it should render `memo_edit_form` template.

## Solution
Change the `get_memo_edit_form` handler to return `memo_edit_form` instead of `memo_fragment`.

## Changes Needed

### File: src/main.rs
**Location**: Lines 1097-1108 in the `get_memo_edit_form` function

**Before**:
```rust
state.templates.render("memo_fragment", &json!({
    "id": id_v,
    "content": content,
    "content_html": content_html,
    "created_at": created_at,
    "created_at_relative": created_at_relative,
    "visibility": visibility,
    "username": user.1,
    "avatar": avatar,
    "resources": resources,
    "tags": tags,
})).into_response()
```

**After**:
```rust
state.templates.render("memo_edit_form", &json!({
    "id": id_v,
    "content": content,
    "visibility": visibility,
})).into_response()
```

## Verification Steps
1. Build the project: `cargo build`
2. Start the application
3. Create a note
4. Click the Edit button on a note
5. Verify that:
   - The Tiptap editor loads with the existing note content
   - Formatting works (bold, italic, etc.)
   - File attachments work
   - Save button updates the note correctly
   - Cancel button returns to view mode

## Context
This fix assumes the following implementation is already in place:
- `get_memo_edit_form` handler exists at `GET /memos/:id/edit`
- Route is registered: `.route("/memos/:id/edit", get(get_memo_edit_form))`
- `MEMO_EDIT_FORM` template contains the Tiptap-powered editor
- `MEMO_FRAGMENT` template has empty edit div: `<div class="memo-edit hidden" id="memo-edit-{{ id }}"></div>`
- JavaScript functions `editMemo(id)` and `cancelEdit(id)` handle HTMX requests and Tiptap lifecycle