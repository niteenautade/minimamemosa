# AGENTS.md — Full Technical Specifications & Feature Blueprint

**Project Name:** MinimaMemosa  
**Target Environment:** Self-Hosted (Docker)  
**Target Idle RAM Footprint:** ~10MB – 20MB (Strict Ceiling: 100MB)  
**Tech Stack:** Rust (Axum Engine) + Embedded SQLite + HTMX + Tailwind CSS

This document serves as the absolute specification sheet and architectural guide for AI Coding Agents or system contributors. Every feature, endpoint, and optimization strategy detailed below is structured to guarantee execution without exceeding our strict memory limit.

---

## 1. System Architecture & RAM Budgets

To comfortably stay under the 100MB boundary, we completely bypass client-side single-page application (SPA) engines and backend JavaScript runtimes[cite: 1]. Instead, we use Server-Side Rendered (SSR) HTML fragments streamed directly via HTMX[cite: 1].

| Layer Component | Implementation Standard | Target Idle Memory |
| :--- | :--- | :--- |
| **Backend Framework** | Rust + `axum` + `tokio` (Minimal features) | ~5.0 MB[cite: 1] |
| **Database Engine** | Embedded SQLite (`rusqlite` with WAL mode & low cache) | ~1.5 MB[cite: 1] |
| **Frontend Engine** | HTMX + Inline Minified Tailwind CSS | 0.0 MB (Server-side)[cite: 1] |
| **Container OS Layer** | Minimal Alpine Linux Base Execution | ~0.5 MB[cite: 1] |

---

## 2. Core Feature Specifications

### A. Register & Login System
* **Mechanics:** User registration and sessions are processed server-side via Axum using `bcrypt` for secure password hashing[cite: 1].
* **Low-RAM Strategy:** To avoid memory-bloating session tracking tables in RAM, session tokens are persisted inside standard client-side `HTTP-Only` cookies using encrypted or securely signed payloads[cite: 1].

### B. Sidebar Note Index
* **Mechanics:** A dedicated vertical panel displays a chronological list of note summaries or titles.
* **HTMX Hook:** The sidebar updates automatically using an HTMX trigger listener (`hx-trigger="memoUpdated from:body"`). When a note is created or updated anywhere in the main view, a tiny HTML fragment swaps into the sidebar DOM container dynamically via `hx-swap="outerHTML"`[cite: 1].

### C. Memos-Style Timeline
* **Mechanics:** Notes are displayed down a clean, scrollable, linear visual timeline sequence[cite: 1].
* **HTMX Hook:** When a user posts a memo, the form sends an asynchronous `hx-post="/memos"` request[cite: 1]. The server returns *only* the single newly generated note fragment, and HTMX instantly prepends it to the layout stack via `hx-swap="afterbegin"`[cite: 1].

### D. Zero-Overhead Light / Dark Mode Toggle
* **Mechanics:** Handled entirely on the client side to avoid consuming server memory resources[cite: 1]. 
* **Implementation:** Theme settings are stored directly within the browser's `localStorage` matrix, toggling utility classes on the root HTML element[cite: 1].

---

## 3. Base Code & Configuration Blueprint

### `Cargo.toml` (Release Optimization Profile)
Compile-time flags must be configured to strip away heavy debugging symbols, optimize for file size, and enforce immediate deallocations[cite: 1].

```toml
[package]
name = "minimamemosa"
version = "0.1.0"
edition = "2021"

[dependencies]
axum = { version = "0.7", features = ["macros"] }
tokio = { version = "1.0", features = ["rt-multi-thread", "macros"] }
rusqlite = { version = "0.31", features = ["bundled"] }
bcrypt = "0.15"
tower-http = { version = "0.5", features = ["fs"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
minijinja = "1.0"

[profile.release]
opt-level = "z"     # Optimize explicitly for smallest binary footprint size
lto = true          # Enable Link-Time Optimization
codegen-units = 1   # Reduce parallel code generation for better optimizations
panic = "abort"     # Strip exception handling unwinding code
strip = true        # Automatically strip symbols from the binary
