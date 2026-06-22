use axum::response::Html;
use minijinja::Environment;

const BASE_TEMPLATE: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>MinimaMemosa</title>
    <script src="/static/tailwindcss.js"></script>
    <script>
        tailwind.config = {
            darkMode: 'class',
            theme: {
                extend: {
                    fontFamily: {
                        sans: ['ui-sans-serif', 'system-ui', '-apple-system', 'BlinkMacSystemFont', '"Segoe UI"', 'Roboto', '"Helvetica Neue"', 'Arial', '"Noto Sans"', 'sans-serif'],
                        mono: ['ui-monospace', 'SFMono-Regular', 'Menlo', 'Monaco', 'Consolas', '"Liberation Mono"', '"Courier New"', 'monospace'],
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
    </script>
    <script src="/static/htmx.min.js"></script>
    <script src="/static/turndown.js"></script>
    <link rel="stylesheet" href="/static/github-dark-dimmed.min.css" id="hljs-theme">
    <script src="/static/highlight.min.js"></script>
    <script src="/static/tiptap-bundle.min.js"></script>
    <script>
        if (localStorage.getItem('theme') === 'dark' || (!localStorage.getItem('theme') && window.matchMedia('(prefers-color-scheme: dark)').matches)) {
            document.documentElement.classList.add('dark');
        }
        function toggleTheme() {
            document.documentElement.classList.toggle('dark');
            var isDark = document.documentElement.classList.contains('dark');
            localStorage.setItem('theme', isDark ? 'dark' : 'light');
            document.getElementById('hljs-theme').href = isDark 
                ? "/static/github-dark-dimmed.min.css" 
                : "/static/github.min.css";
        }
        document.addEventListener("DOMContentLoaded", () => {
            var isDark = document.documentElement.classList.contains('dark');
            document.getElementById('hljs-theme').href = isDark 
                ? "/static/github-dark-dimmed.min.css" 
                : "/static/github.min.css";
        });
        document.addEventListener('htmx:afterSwap', function(evt) {
            document.querySelectorAll('pre code').forEach((block) => {
                hljs.highlightElement(block);
            });
        });
        document.addEventListener('DOMContentLoaded', function() {
            document.querySelectorAll('pre code').forEach((block) => {
                hljs.highlightElement(block);
            });
        });
    </script>
    <style>
        :root {
            --bg: oklch(0.9818 0.0054 95.0986);
            --fg: oklch(0.2438 0.0269 95.7226);
            --card: oklch(1 0 0);
            --card-fg: oklch(0.1908 0.002 106.5859);
            --border: oklch(0.8847 0.0069 97.3627);
            --muted: oklch(0.9341 0.0153 90.239);
            --muted-fg: oklch(0.5559 0.0075 97.4233);
            --sidebar: oklch(0.9663 0.008 98.8792);
            --sidebar-fg: oklch(0.359 0.0051 106.6524);
        }
        .dark {
            --bg: oklch(0.24 0.008 255);
            --fg: oklch(0.9 0.006 255);
            --card: oklch(0.275 0.009 255);
            --card-fg: oklch(0.9 0.006 255);
            --border: oklch(0.38 0.01 255);
            --muted: oklch(0.35 0.011 255);
            --muted-fg: oklch(0.72 0.007 255);
            --sidebar: oklch(0.21 0.009 255);
            --sidebar-fg: oklch(0.76 0.007 255);
        }
        body { font-family: var(--font-sans, ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, "Noto Sans", sans-serif); }
        .memo-content h1 { font-size: 1.5rem; font-weight: 700; margin-bottom: 0.5rem; border-bottom: 1px solid var(--border); padding-bottom: 0.25rem; }
        .memo-content h2 { font-size: 1.25rem; font-weight: 600; margin-bottom: 0.5rem; }
        .memo-content h3 { font-size: 1.125rem; font-weight: 600; margin-bottom: 0.25rem; }
        .memo-content p { margin-bottom: 0.5rem; line-height: 1.625; }
        .memo-content p:last-child { margin-bottom: 0; }
        .memo-content ul, .memo-content ol { padding-left: 1.5rem; margin-bottom: 0.5rem; }
        .memo-content li { list-style: disc; margin-bottom: 0.25rem; }
        .memo-content ol li { list-style: decimal; }
        .memo-content code { background: var(--muted); padding: 0.125rem 0.375rem; border-radius: 0.25rem; font-size: 0.875rem; color: var(--fg); font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace; }
        .memo-content pre { background: var(--muted); padding: 0.75rem; border-radius: 0.5rem; overflow-x: auto; margin-bottom: 0.75rem; border: 1px solid var(--border); font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace; }
        .memo-content pre code { background: none; padding: 0; color: inherit; }
        .memo-content blockquote { border-left: 3px solid var(--border); padding-left: 0.75rem; margin: 0.5rem 0; color: var(--muted-fg); }
        .memo-content a { color: oklch(0.45 0.08 250); text-decoration: none; }
        .memo-content a:hover { text-decoration: underline; }
        .dark .memo-content a { color: oklch(0.62 0.11 250); }
        .memo-content hr { border: none; border-top: 1px solid var(--border); margin: 0.75rem 0; }
        .memo-content table { border-collapse: collapse; margin-bottom: 0.5rem; width: 100%; }
        .memo-content th, .memo-content td { border: 1px solid var(--border); padding: 0.375rem 0.75rem; text-align: left; }
        .memo-content th { background: var(--muted); font-weight: 600; }
        .memo-content img { max-width: 100%; border-radius: 0.5rem; margin: 0.5rem 0; }
        .memo-content ul.task-list { list-style: none; padding-left: 0; }
        .memo-content li.task-list-item { display: flex; align-items: flex-start; gap: 0.375rem; }
        .memo-content li.task-list-item input[type="checkbox"] { margin-top: 0.25rem; }
        .avatar-initials { display: inline-flex; align-items: center; justify-content: center; width: 2rem; height: 2rem; border-radius: 9999px; font-size: 0.875rem; font-weight: 600; flex-shrink: 0; }
        .avatar-initials-sm { width: 1.5rem; height: 1.5rem; font-size: 0.75rem; }
        .auto-expand-textarea { min-height: 2.5rem; overflow: hidden; resize: none; }
        .memo-editor { transition: border-color 0.15s ease; }
        .memo-editor:focus-within { border-color: oklch(0.45 0.08 250); }
        .dark .memo-editor:focus-within { border-color: oklch(0.62 0.11 250); }
        @keyframes shimmer {
            0% { background-position: -200% 0; }
            100% { background-position: 200% 0; }
        }
        .shimmer-bg {
            background: linear-gradient(90deg, var(--muted) 25%, var(--border) 50%, var(--muted) 75%);
            background-size: 200% 100%;
            animation: shimmer 1.5s infinite linear;
        }
        .tiptap-editor[contenteditable="false"] {
            pointer-events: none;
            cursor: wait;
        }
        .tiptap-editor .ProseMirror { outline: none; white-space: pre-wrap; word-wrap: break-word; min-height: 4rem; }
        .tiptap-editor .ProseMirror p { margin: 0.25rem 0; }
        .tiptap-editor .ProseMirror h1 { font-size: 1.5rem; font-weight: 700; margin: 0.25rem 0; border-bottom: 1px solid var(--border); padding-bottom: 0.25rem; }
        .tiptap-editor .ProseMirror h2 { font-size: 1.25rem; font-weight: 600; margin: 0.25rem 0; }
        .tiptap-editor .ProseMirror h3 { font-size: 1.125rem; font-weight: 600; margin: 0.25rem 0; }
        .tiptap-editor .ProseMirror ul, .tiptap-editor .ProseMirror ol { padding-left: 1.25rem; margin: 0.25rem 0; }
        .tiptap-editor .ProseMirror li { list-style: disc; }
        .tiptap-editor .ProseMirror ol li { list-style: decimal; }
        .tiptap-editor .ProseMirror code { background: var(--muted); padding: 0.125rem 0.25rem; border-radius: 0.25rem; font-size: 0.875rem; }
        .tiptap-editor .ProseMirror pre { background: var(--muted); padding: 0.5rem; border-radius: 0.375rem; margin: 0.25rem 0; }
        .tiptap-editor .ProseMirror pre code { background: none; padding: 0; }
        .tiptap-editor .ProseMirror blockquote { border-left: 2px solid var(--border); padding-left: 0.5rem; margin: 0.25rem 0; color: var(--muted-fg); }
        .tiptap-editor .ProseMirror a { color: oklch(0.45 0.08 250); }
        .dark .tiptap-editor .ProseMirror a { color: oklch(0.62 0.11 250); }
        .tiptap-editor .ProseMirror hr { border: none; border-top: 1px solid var(--border); margin: 0.75rem 0; }
        .tiptap-editor .ProseMirror table { border-collapse: collapse; margin: 0.25rem 0; width: 100%; }
        .tiptap-editor .ProseMirror th, .tiptap-editor .ProseMirror td { border: 1px solid var(--border); padding: 0.25rem 0.5rem; text-align: left; }
        .tiptap-editor .ProseMirror th { background: var(--muted); font-weight: 600; }
        .tiptap-editor .ProseMirror img { display: block; max-width: 100%; max-height: calc((100vw - 18rem) / 4); height: auto; object-fit: contain; border-radius: 0.5rem; margin: 0.5rem 0; }
        .tiptap-editor .ProseMirror p.is-editor-empty:first-child::before { content: attr(data-placeholder); color: var(--muted-fg); pointer-events: none; float: left; height: 0; }
        .tiptap-editor[data-empty="true"]:not(:has(.ProseMirror)):before { content: attr(data-placeholder); color: var(--muted-fg); pointer-events: none; }
        * { scrollbar-width: thin; scrollbar-color: color-mix(in srgb, var(--muted-fg) 45%, transparent) transparent; }
        #image-modal { background: rgba(0,0,0,0.85); }
        #image-modal img { max-width: 95vw; max-height: 95vh; }
    </style>
</head>
<body class="bg-background text-foreground min-h-screen">
    {% block content %}{% endblock %}
<div id="image-modal" class="fixed inset-0 z-[9999] flex items-center justify-center hidden" onclick="closeImageModal()">
    <button class="absolute top-4 right-4 text-white/70 hover:text-white text-2xl leading-none w-10 h-10 flex items-center justify-center rounded-full bg-black/20 hover:bg-black/40 transition-colors">&times;</button>
    <img src="" alt="Fullscreen image" class="object-contain rounded-lg shadow-2xl" onclick="event.stopPropagation()">
</div>
</body>
</html>"#;

const LOGIN_TEMPLATE: &str = r#"{% extends "base" %}
{% block content %}
<div class="flex items-center justify-center min-h-screen">
    <div class="w-full max-w-sm mx-4">
        <h1 class="text-2xl font-bold mb-6 text-center">MinimaMemosa</h1>
        <form action="/login" method="post" class="space-y-4">
            <div>
                <label for="username" class="block text-sm font-medium mb-1">Username</label>
                <input type="text" name="username" id="username" required
                    class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-card focus:outline-none focus:ring-2 focus:ring-blue-500">
            </div>
            <div>
                <label for="password" class="block text-sm font-medium mb-1">Password</label>
                <input type="password" name="password" id="password" required
                    class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-card focus:outline-none focus:ring-2 focus:ring-blue-500">
            </div>
            {% if error %}
            <p class="text-red-500 text-sm">{{ error }}</p>
            {% endif %}
            <button type="submit"
                class="w-full py-2 px-4 bg-blue-600 text-white rounded-lg hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500">
                Sign In
            </button>
        </form>
        <p class="text-center text-sm text-gray-500 mt-4">
            Don't have an account? <a href="/register" class="text-blue-600 hover:underline">Register</a>
        </p>
    </div>
</div>
{% endblock %}"#;

const REGISTER_TEMPLATE: &str = r#"{% extends "base" %}
{% block content %}
<div class="flex items-center justify-center min-h-screen">
    <div class="w-full max-w-sm mx-4">
        <h1 class="text-2xl font-bold mb-6 text-center">Create Account</h1>
        <form action="/register" method="post" class="space-y-4">
            <div>
                <label for="username" class="block text-sm font-medium mb-1">Username</label>
                <input type="text" name="username" id="username" required
                    class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-card focus:outline-none focus:ring-2 focus:ring-blue-500">
            </div>
            <div>
                <label for="password" class="block text-sm font-medium mb-1">Password</label>
                <input type="password" name="password" id="password" required
                    class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-card focus:outline-none focus:ring-2 focus:ring-blue-500">
            </div>
            <div class="space-y-2">
                <div class="flex items-center justify-center p-2 bg-white dark:bg-gray-800 rounded-lg border border-border">
                    <img src="{{ captcha_image }}" alt="Captcha" class="h-16 object-contain" />
                </div>
                <div>
                    <label for="captcha_answer" class="block text-sm font-medium mb-1">Verify Captcha Code</label>
                    <input type="text" name="captcha_answer" id="captcha_answer" required placeholder="Enter code above"
                        class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-card focus:outline-none focus:ring-2 focus:ring-blue-500">
                </div>
            </div>
            {% if error %}
            <p class="text-red-500 text-sm">{{ error }}</p>
            {% endif %}
            <button type="submit"
                class="w-full py-2 px-4 bg-blue-600 text-white rounded-lg hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500">
                Register
            </button>
        </form>
        <p class="text-center text-sm text-gray-500 mt-4">
            Already have an account? <a href="/login" class="text-blue-600 hover:underline">Sign in</a>
        </p>
    </div>
</div>
{% endblock %}"#;

const TIMELINE_TEMPLATE: &str = r##"{% extends "base" %}
{% block content %}
<div class="flex h-screen overflow-hidden">
    <!-- Icon Bar -->
    <div class="w-14 flex-shrink-0 bg-card border-r border-border flex flex-col items-center py-3 gap-2 z-20">
        <button id="icon-timeline"
            onclick="location.hash='timeline'"
            class="p-2.5 rounded-xl bg-blue-100 dark:bg-blue-900/30 text-blue-600 dark:text-blue-400 transition-colors"
            title="Timeline">
            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <rect x="3" y="4" width="18" height="16" rx="2" stroke-width="2"/>
                <line x1="8" y1="10" x2="16" y2="10" stroke-width="2"/>
                <line x1="8" y1="14" x2="14" y2="14" stroke-width="2"/>
            </svg>
        </button>
        <button id="icon-notes"
            onclick="location.hash='notes'"
            class="p-2.5 rounded-xl text-muted-fg hover:bg-muted hover:text-foreground transition-colors"
            title="Notes">
            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" stroke-width="2"/>
                <polyline points="14 2 14 8 20 8" stroke-width="2"/>
                <line x1="12" y1="18" x2="12" y2="12" stroke-width="2"/>
                <line x1="9" y1="15" x2="15" y2="15" stroke-width="2"/>
            </svg>
        </button>
        <button id="icon-resources"
            onclick="location.hash='resources'"
            class="p-2.5 rounded-xl text-muted-fg hover:bg-muted hover:text-foreground transition-colors"
            title="Resources">
            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16l4.586-4.586a2 2 0 012.828 0L16 16m-2-2l1.586-1.586a2 2 0 012.828 0L20 14m-6-6h.01M6 20h12a2 2 0 002-2V6a2 2 0 00-2-2H6a2 2 0 00-2 2v12a2 2 0 002 2z"/>
            </svg>
        </button>
    </div>

    <!-- Sidebar Panel (timeline view - search + calendar) -->
    <div id="sidebar-panel"
        class="w-72 flex-shrink-0 bg-sidebar border-r border-border flex-col h-full overflow-hidden">
        <div id="sidebar-content"
            hx-trigger="load once"
            hx-get="/sidebar-timeline"
            hx-swap="innerHTML"
            class="flex flex-col h-full">
        </div>
    </div>

    <!-- Notes Panel -->
    <div id="notes-panel"
        class="w-72 flex-shrink-0 bg-sidebar border-r border-border flex-col h-full hidden"
        hx-trigger="memoUpdated from:body"
        hx-get="/notes-panel"
        hx-swap="innerHTML">
    </div>

    <!-- Resources Panel -->
    <div id="resources-panel"
        class="w-72 flex-shrink-0 bg-sidebar border-r border-border flex-col h-full hidden"
        hx-trigger="load once"
        hx-get="/resources-feed"
        hx-swap="innerHTML">
    </div>

    <!-- Main Content -->
    <div id="main-content" class="flex-1 flex flex-col h-full overflow-hidden min-w-0">
        <!-- Header -->
        <header class="flex items-center justify-between px-6 py-2.5 border-b border-border bg-white dark:bg-gray-900 flex-shrink-0">
            <div class="flex items-center gap-3">
                <span class="text-sm font-semibold text-card-fg">MinimaMemosa</span>
            </div>
            <div class="flex items-center gap-2">
                <button onclick="toggleTheme()"
                    class="p-1.5 rounded-lg hover:bg-muted text-muted-fg transition-colors">
                    <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M20.354 15.354A9 9 0 018.646 3.646 9.003 9.003 0 0012 21a9.003 9.003 0 008.354-5.646z"/>
                    </svg>
                </button>
                <div class="flex items-center gap-2 px-2 py-1 rounded-lg hover:bg-muted cursor-pointer">
                    <div class="avatar-initials avatar-initials-sm bg-blue-500 text-white">{{ avatar }}</div>
                    <span class="text-sm text-card-fg">{{ username }}</span>
                </div>
                <a href="/logout" class="text-xs text-gray-400 hover:text-red-500 transition-colors ml-1" title="Logout">
                    <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17 16l4-4m0 0l-4-4m4 4H7m6 4v1a3 3 0 01-3 3H6a3 3 0 01-3-3V7a3 3 0 013-3h4a3 3 0 013 3v1"/>
                    </svg>
                </a>
            </div>
        </header>

        <!-- Timeline View -->
        <div id="timeline-view" class="flex-1 flex flex-col overflow-hidden">
            <div class="flex-1 overflow-y-auto px-6 py-5">
                <div class="max-w-2xl mx-auto">
                    <!-- Memos-style Editor Card -->
                    <form id="memo-form" hx-post="/memos"
                          hx-swap="afterbegin"
                          hx-target="#timeline"
                          hx-on::after-request="if(event.detail.successful){resetEditor();htmx.trigger('body','memoUpdated')}"
                          class="memo-editor mb-6 bg-card border border-border rounded-xl shadow-sm"
                          ondragover="event.preventDefault(); this.classList.add('border-blue-500')"
                          ondragleave="event.preventDefault(); this.classList.remove('border-blue-500')"
                           ondrop="event.preventDefault(); this.classList.remove('border-blue-500'); handleDrop(event)"
                            onsubmit="document.getElementById('memo-editor-input').value = getTiptapMarkdown();">
                         <div class="px-4 pt-3 pb-1 relative">
                              <div id="memo-editor"
                                 class="w-full bg-transparent text-foreground text-base leading-relaxed min-h-[6rem] tiptap-editor animate-pulse bg-muted/30 rounded shimmer-bg"
                                 contenteditable="false"
                                 data-placeholder="What's on your mind..."
                                 oninput="onFallbackInput(this)"
                                 onkeydown="onFallbackKeydown(event, this)"></div>
                             <!-- Attachment Previews -->
                              <div id="attachment-preview-container" class="border border-border rounded-xl bg-card overflow-hidden hidden">
                                 <div id="attachment-preview-list" class="flex flex-col"></div>
                             </div>
                             <input type="hidden" name="content" id="memo-editor-input" value="">
                         </div>
                         <!-- Slash Commands Dropdown -->
                         <div id="slash-menu" class="hidden absolute left-4 bottom-14 bg-card border border-border rounded-lg shadow-lg py-1 min-w-[200px] z-50"></div>
                         <input type="file" id="image-upload-input" accept="image/*" multiple class="hidden" onchange="uploadFilesForEditor(this.files);this.value=''">
                         <input type="file" id="file-upload-input" accept="*/*" multiple class="hidden" onchange="uploadFilesForEditor(this.files);this.value=''">
                         <div class="flex items-center justify-between px-4 py-2 border-t border-border">
                             <div class="flex items-center gap-1">
                                 <!-- Emoji Picker -->
                                 <div class="relative">
                                     <button type="button" onclick="toggleEmojiPicker()" class="p-1.5 rounded-md text-muted-fg hover:text-foreground hover:bg-muted transition-colors" title="Insert Emoji">
                                         <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M14.828 14.828a4 4 0 01-5.656 0M9 10h.01M15 10h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"/></svg>
                                     </button>
                                     <div id="emoji-picker" class="hidden absolute top-full left-0 mt-1 bg-card border border-border rounded-xl shadow-xl p-2 z-50 w-[280px] max-h-[200px] overflow-y-auto">
                                         <div id="emoji-grid" class="grid grid-cols-7 gap-0.5 text-lg"></div>
                                     </div>
                                 </div>
                                 <!-- Plus Menu -->
                                 <div class="relative">
                                     <button type="button" onclick="togglePlusMenu()" class="p-1.5 rounded-md text-muted-fg hover:text-foreground hover:bg-muted transition-colors" title="More">
                                         <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4"/></svg>
                                     </button>
                                      <div id="plus-menu" class="hidden absolute top-full left-0 mt-1 bg-card border border-border rounded-xl shadow-xl py-1 z-50 min-w-[180px]">
                                         <button type="button" onclick="uploadImage()" class="flex items-center gap-2 w-full px-3 py-1.5 text-xs text-foreground hover:bg-muted transition-colors">
                                             <span>📷</span> Upload Image
                                         </button>
                                         <button type="button" onclick="uploadFile()" class="flex items-center gap-2 w-full px-3 py-1.5 text-xs text-foreground hover:bg-muted transition-colors">
                                             <span>📎</span> Upload File
                                         </button>
                                         <button type="button" id="record-audio-btn" onclick="toggleAudioRecording()" class="flex items-center gap-2 w-full px-3 py-1.5 text-xs text-foreground hover:bg-muted transition-colors">
                                             <span>🎤</span><span id="record-label">Record Audio</span>
                                         </button>
                                         <button type="button" onclick="toggleLinkMemo()" class="flex items-center gap-2 w-full px-3 py-1.5 text-xs text-foreground hover:bg-muted transition-colors">
                                             <span>🔗</span> Link Memo
                                         </button>
                                     </div>
                                     <!-- Link Memo Search Dropdown -->
                                      <div id="link-memo-dropdown" class="hidden absolute top-full left-0 mt-1 bg-card border border-border rounded-xl shadow-xl z-50 w-[250px]">
                                         <div class="p-2">
                                             <input type="text" id="link-memo-search" placeholder="Search memos..." oninput="searchLinkMemos(this.value)" class="w-full px-2 py-1.5 text-xs bg-muted border border-border rounded-lg focus:outline-none focus:ring-1 focus:ring-blue-500">
                                         </div>
                                         <div id="link-memo-results" class="max-h-[200px] overflow-y-auto"></div>
                                     </div>
                                 </div>
                                 <div class="visibility-dropdown relative" data-vis="private">
                                     <button type="button" onclick="toggleVisDropdown(this)" class="flex items-center gap-1 px-1.5 py-1 rounded-md text-muted-fg hover:text-foreground hover:bg-muted transition-colors text-xs">
                                         <span class="vis-label flex items-center gap-1"><svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z"/></svg>Private</span>
                                         <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7"/></svg>
                                     </button>
                                     <div class="vis-dropdown-menu hidden absolute top-full left-0 mt-1 bg-card border border-border rounded-lg shadow-lg py-1 min-w-[140px] z-50">
                                         <button type="button" data-vis-value="public" onclick="selectVis(this)" class="flex items-center gap-2 w-full px-3 py-1.5 text-xs text-foreground hover:bg-muted transition-colors">
                                             <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3.055 11H5a2 2 0 012 2v1a2 2 0 002 2 2 2 0 012 2v2.945M8 3.935V5.5A2.5 2.5 0 0010.5 8h.5a2 2 0 012 2 2 2 0 104 0 2 2 0 012-2h1.064M15 20.488V18a2 2 0 012-2h3.064M21 12a9 9 0 11-18 0 9 9 0 0118 0z"/></svg>
                                             Public
                                         </button>
                                         <button type="button" data-vis-value="protected" onclick="selectVis(this)" class="flex items-center gap-2 w-full px-3 py-1.5 text-xs text-foreground hover:bg-muted transition-colors">
                                             <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><rect x="3" y="11" width="18" height="11" rx="2" stroke-width="2"/><path d="M7 11V7a5 5 0 0110 0v4" stroke-width="2"/></svg>
                                             Protected
                                         </button>
                                         <button type="button" data-vis-value="private" onclick="selectVis(this)" class="flex items-center gap-2 w-full px-3 py-1.5 text-xs text-foreground hover:bg-muted transition-colors">
                                             <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z"/></svg>
                                             Private
                                         </button>
                                     </div>
                                     <input type="hidden" name="visibility" value="private">
                                 </div>
                                 <span class="text-xs text-muted-fg">Ctrl+Enter</span>
                             </div>
                             <button type="submit" id="save-memo-btn" disabled
                                 class="py-1.5 px-4 bg-blue-600 hover:bg-blue-700 text-white text-sm font-medium rounded-lg transition-colors focus:outline-none focus:ring-2 focus:ring-blue-500 disabled:opacity-50 disabled:cursor-not-allowed disabled:hover:bg-blue-600">
                                 Save
                             </button>
                         </div>
                    </form>

                    <!-- Timeline -->
                    <div id="timeline" class="space-y-1"
                        hx-trigger="memoUpdated from:body"
                        hx-get="/memos-feed"
                        hx-swap="innerHTML">
                        {% for group in memo_groups %}
                        <div class="mb-4">
                            <div class="flex items-center gap-2 mb-3">
                                <span class="text-xs font-medium text-muted-fg uppercase tracking-wider">{{ group.label }}</span>
                                <span class="text-xs text-muted-fg">{{ group.date }}</span>
                                <div class="flex-1 border-t border-gray-100 dark:border-gray-700"></div>
                                <span class="text-xs text-muted-fg font-mono">{{ group.memos|length }}</span>
                            </div>
                            <div class="space-y-2">
                                {% for memo in group.memos %}
                                {% set id = memo.id %}
                                {% set content = memo.content %}
                                {% set content_html = memo.content_html %}
                                {% set visibility = memo.visibility %}
                                {% set created_at = memo.created_at %}
                                {% set created_at_relative = memo.created_at_relative %}
                                {% set resources = memo.resources %}
                                {% set tags = memo.tags %}
                                {% include "memo_fragment" %}
                                {% endfor %}
                            </div>
                        </div>
                        {% endfor %}
                        {% if not memo_groups %}
                        <div class="text-center py-16">
                            <p class="text-muted-fg text-sm">No memos yet. Write your first memo above!</p>
                        </div>
                        {% endif %}
                    </div>
                </div>
            </div>
        </div>

        <!-- Note Detail View (hidden by default) -->
        <div id="note-detail-view" class="hidden flex-1 flex-col overflow-y-auto px-6 py-4">
        </div>
    </div>
</div>

<script>
    /* ── Editor Fallback (works if Tiptap CDN fails) ── */
    function getEditorText() {
        var el = document.getElementById('memo-editor');
        if (window.tiptapEditor) {
            return window.tiptapEditor.getText();
        }
        return el ? (el.innerText || el.textContent || '') : '';
    }
    function onFallbackInput(el) {
        var text = getEditorText();
        document.getElementById('memo-editor-input').value = text;
        var isEmpty = !text.trim();
        if (isEmpty) {
            el.setAttribute('data-empty', 'true');
        } else {
            el.removeAttribute('data-empty');
        }
        updateSaveButtonState();
    }
    function onFallbackKeydown(e, el) {
        if ((e.ctrlKey || e.metaKey) && e.key === 'Enter') {
            e.preventDefault();
            document.getElementById('memo-editor-input').value = getEditorText();
            var btn = document.getElementById('memo-form').querySelector('button[type="submit"]');
            if (btn) btn.click();
            return;
        }
        if (e.key === 'Enter' && !e.shiftKey) {
            e.preventDefault();
            document.execCommand('insertLineBreak');
            return;
        }
        if (e.key === '/') {
            setTimeout(function() { checkSlashCommand(el); }, 0);
            return;
        }
        if (e.key === 'Escape') {
            document.getElementById('slash-menu').classList.add('hidden');
        }
    }
    function getTextOffset(el) {
        var sel = window.getSelection();
        if (!sel.rangeCount || !el.contains(sel.anchorNode)) return getEditorText().length;
        var range = sel.getRangeAt(0);
        var offset = 0;
        var walker = document.createTreeWalker(el, NodeFilter.SHOW_TEXT, null, false);
        while (walker.nextNode()) {
            if (walker.currentNode === range.startContainer) { offset += range.startOffset; break; }
            offset += walker.currentNode.textContent.length;
        }
        return offset;
    }
    function restoreCursor(el, target) {
        el.focus();
        var walker = document.createTreeWalker(el, NodeFilter.SHOW_TEXT, null, false);
        var node, pos = 0;
        while (node = walker.nextNode()) {
            var len = node.textContent.length;
            if (pos + len >= target) {
                var r = document.createRange();
                r.setStart(node, target - pos);
                r.collapse(true);
                var s = window.getSelection();
                s.removeAllRanges(); s.addRange(r);
                return;
            }
            pos += len;
        }
        var r = document.createRange();
        r.selectNodeContents(el); r.collapse(false);
        var s = window.getSelection();
        s.removeAllRanges(); s.addRange(r);
    }

    /* ── Tiptap Editor Globals ── */
    function getTiptapMarkdown() {
        var ed = window.tiptapEditor;
        if (ed) {
            if (ed.isEmpty) return '';
            if (ed.storage.markdown) {
                return ed.storage.markdown.getMarkdown();
            }
            var html = ed.getHTML();
            if (html && html !== '<p></p>') {
                try {
                    var ts = new TurndownService({ headingStyle: 'atx' });
                    return ts.turndown(html);
                } catch(e) {}
            }
            return '';
        }
        return getEditorText();
    }
    function updateSaveButtonState() {
        var btn = document.getElementById('save-memo-btn');
        if (!btn) return;
        btn.disabled = getTiptapMarkdown().trim() === '' && editorAttachments.length === 0;
    }
    function resetEditor() {
        var ed = window.tiptapEditor;
        if (ed) {
            ed.commands.clearContent(true);
        } else {
            var el = document.getElementById('memo-editor');
            if (el) { el.innerHTML = ''; el.setAttribute('data-empty', 'true'); }
        }
        var input = document.getElementById('memo-editor-input');
        if (input) input.value = '';
        editorAttachments = [];
        renderEditorAttachments();
        var dd = document.querySelector('.vis-dropdown-menu');
        if (dd) dd.classList.add('hidden');
        updateSaveButtonState();
    }
    function insertContenteditable(text) {
        var ed = window.tiptapEditor;
        if (ed) { ed.chain().focus().insertContent(text).run(); }
        else {
            var el = document.getElementById('memo-editor');
            if (!el) return;
            var sel = window.getSelection();
            if (!sel.rangeCount) { el.focus(); sel.selectNodeContents(el); sel.collapse(false); }
            var range = sel.getRangeAt(0);
            range.deleteContents();
            range.insertNode(document.createTextNode(text));
            range.collapse(false);
            sel.removeAllRanges(); sel.addRange(range);
            document.getElementById('memo-editor-input').value = getEditorText();
            updateSaveButtonState();
        }
    }

    /* ── Emoji Picker ── */
    var EMOJIS = ['😀','😁','😂','🤣','😃','😄','😅','😆','😉','😊','😋','😎','😍','🥰','😘','😗','😙','😚','🙂','🤗','🤩','🤔','🤨','😐','😑','😶','🙄','😏','😣','😥','😮','🤐','😯','😪','😫','😴','😌','😛','😜','😝','🤤','😒','😓','😔','😕','🙃','🤑','😲','☹️','🙁','😖','😞','😟','😤','😢','😭','😦','😧','😨','😩','🤯','😬','😰','😱','🥵','🥶','😳','🤪','😵','😡','😠','🤬','👍','👎','👊','✊','🤛','🤜','👏','🙌','👐','🤲','🤝','🙏','✌️','🤟','🤘','👌','💪','❤️','🧡','💛','💚','💙','💜','🖤','💔','💕','💞','💗','💖','✨','🔥','⭐','🌟','💡','📝','📌','📍','🎉','🎊','🎈','🎁','🏆','✅','❌','💯','🔗','♻️','🔄','📎','🔒','🔓','☀️','🌙','⭐','🌈','⚡','🌊','🔥','❄️','🌱','🌿','🍀','🎯','🚀','💻','📱','⌨️','🖥️','📷','🎥','🔊','📢','💬','🗨️','👀','🖐️','✋','🤚','👋'];
    var audioRecorder = null;
    var audioChunks = [];
    function toggleEmojiPicker() {
        var picker = document.getElementById('emoji-picker');
        if (picker.classList.contains('hidden')) {
            var grid = document.getElementById('emoji-grid');
            if (!grid.children.length) {
                grid.innerHTML = EMOJIS.map(function(e) { return '<button type="button" onclick="insertEmoji(\''+e+'\')" class="hover:bg-muted rounded p-0.5 transition-colors">'+e+'</button>'; }).join('');
            }
            closeAllDropdowns();
            picker.classList.remove('hidden');
        } else {
            picker.classList.add('hidden');
        }
    }
    function insertEmoji(emoji) { insertContenteditable(emoji); document.getElementById('emoji-picker').classList.add('hidden'); }
    function togglePlusMenu() {
        var menu = document.getElementById('plus-menu');
        if (menu.classList.contains('hidden')) {
            closeAllDropdowns();
            menu.classList.remove('hidden');
        } else { menu.classList.add('hidden'); }
    }
    function uploadImage() { document.getElementById('image-upload-input').click(); closeAllDropdowns(); }
    function uploadFile() { document.getElementById('file-upload-input').click(); closeAllDropdowns(); }
    function toggleAudioRecording() {
        var label = document.getElementById('record-label');
        if (audioRecorder && audioRecorder.state === 'recording') {
            audioRecorder.stop();
            label.textContent = 'Record Audio';
            return;
        }
        if (!navigator.mediaDevices || !navigator.mediaDevices.getUserMedia) { alert('Audio recording not supported in this browser.'); return; }
        navigator.mediaDevices.getUserMedia({ audio: true }).then(function(stream) {
            audioChunks = [];
            audioRecorder = new MediaRecorder(stream);
            audioRecorder.ondataavailable = function(e) { if (e.data.size > 0) audioChunks.push(e.data); };
            audioRecorder.onstop = function() {
                stream.getTracks().forEach(function(t) { t.stop(); });
                var blob = new Blob(audioChunks, { type: 'audio/webm' });
                var file = new File([blob], 'recording.webm', { type: 'audio/webm' });
                uploadFilesForEditor([file]);
                audioRecorder = null;
            };
            audioRecorder.start();
            label.textContent = '⏹ Stop Recording';
            closeAllDropdowns();
        }).catch(function() { alert('Microphone access denied.'); });
    }
    function toggleLinkMemo() {
        var dd = document.getElementById('link-memo-dropdown');
        if (dd.classList.contains('hidden')) {
            closeAllDropdowns();
            dd.classList.remove('hidden');
            document.getElementById('link-memo-search').value = '';
            document.getElementById('link-memo-results').innerHTML = '<div class="px-3 py-2 text-xs text-muted-fg">Type to search...</div>';
            setTimeout(function() { document.getElementById('link-memo-search').focus(); }, 100);
        } else { dd.classList.add('hidden'); }
    }
    function searchLinkMemos(query) {
        if (!query || query.length < 1) { document.getElementById('link-memo-results').innerHTML = '<div class="px-3 py-2 text-xs text-muted-fg">Type to search...</div>'; return; }
        fetch('/memos-json?q=' + encodeURIComponent(query))
            .then(function(r) { return r.json(); })
            .then(function(data) {
                var container = document.getElementById('link-memo-results');
                if (!data.length) { container.innerHTML = '<div class="px-3 py-2 text-xs text-muted-fg">No results</div>'; return; }
                container.innerHTML = data.map(function(m) {
                    var title = m.title || m.content || 'Untitled';
                    var preview = (m.content || '').replace(/[\[\]!#*`>]/g, '').substring(0, 80);
                    return '<button type="button" onclick="insertMemoLink(\''+m.id+'\',\''+title.replace(/'/g,"\\'")+'\')" class="flex flex-col items-start w-full px-3 py-1.5 text-xs text-left hover:bg-muted transition-colors"><span class="font-medium text-foreground">'+escapeHtml(title)+'</span><span class="text-muted-fg truncate w-full">'+escapeHtml(preview)+'</span></button>';
                }).join('');
            }).catch(function() {});
    }
    function insertMemoLink(id, title) { insertContenteditable('['+title+'](/memos/'+id+')'); document.getElementById('link-memo-dropdown').classList.add('hidden'); }
    function closeAllDropdowns() {
        ['emoji-picker','plus-menu','link-memo-dropdown'].forEach(function(id) { document.getElementById(id).classList.add('hidden'); });
    }
    function escapeHtml(s) { var d = document.createElement('div'); d.appendChild(document.createTextNode(s)); return d.innerHTML; }

    /* ── Slash Commands (fallback version for contenteditable) ── */
    var FALLBACK_SLASH_COMMANDS = [
        { label: 'Heading 1', insert: '# ', icon: '<svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h7"/></svg>' },
        { label: 'Heading 2', insert: '## ', icon: '<svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h7"/></svg>' },
        { label: 'Bold', insert: '****', icon: '<svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 4h8a4 4 0 014 4 4 4 0 01-4 4H6z"/><path d="M6 12h9a4 4 0 010 8H6z"/></svg>' },
        { label: 'Italic', insert: '**', icon: '<svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 4h6m-2 0l-6 16"/></svg>' },
        { label: 'Bullet List', insert: '- ', icon: '<svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 6h13M8 12h13M8 18h13M3 6h.01M3 12h.01M3 18h.01"/></svg>' },
        { label: 'Numbered List', insert: '1. ', icon: '<svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5h11M9 12h11M9 19h11M5 5v.01M5 12v.01M5 19v.01"/></svg>' },
        { label: 'Code Block', insert: '```\n\n```', icon: '<svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 20l4-16m4 4l4 4-4 4M6 16l-4-4 4-4"/></svg>' },
        { label: 'Blockquote', insert: '> ', icon: '<svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 10h16M4 14h16M4 18h16"/></svg>' },
        { label: 'Todo List', insert: '- [ ] ', icon: '<svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 11l3 3L22 4"/></svg>' },
        { label: 'Table', insert: '| Col1 | Col2 |\n|------|------|\n| Cell | Cell |', icon: '<svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 10h18M3 14h18M3 18h18M3 6h18"/></svg>' },
        { label: 'Code', insert: '``', icon: '<svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 20l4-16m4 4l4 4-4 4M6 16l-4-4 4-4"/></svg>' },
    ];
    function applySlashCommand(cmd) {
        var ed = window.tiptapEditor;
        if (ed) {
            var cursorPos = ed.state.selection.$anchor.pos;
            var textBefore = ed.state.doc.textBetween(0, cursorPos, '\n', '');
            var match = textBefore.match(/(^|\s)\/([a-z]*)$/i);
            if (match) {
                var slashStart = cursorPos - (match[0].length - (match[1] ? match[1].length : 0));
                ed.chain().focus().deleteRange({ from: slashStart, to: cursorPos }).insertContent(cmd.insert).run();
            }
        } else {
            var el = document.getElementById('memo-editor');
            if (!el) return;
            var cursor = getTextOffset(el);
            var text = getEditorText();
            var before = text.substring(0, cursor);
            var slashIdx = before.lastIndexOf('/');
            var prefix = before.substring(0, slashIdx);
            var after = text.substring(cursor);
            var newText = prefix + cmd.insert + after;
            el.innerText = newText;
            var newPos = prefix.length + cmd.insert.length;
            restoreCursor(el, Math.min(newPos, newText.length));
        }
        document.getElementById('slash-menu').classList.add('hidden');
        document.getElementById('memo-editor-input').value = getTiptapMarkdown();
    }
    function checkSlashCommand(el) {
        var text = getEditorText();
        var cursor = getTextOffset(el);
        var before = text.substring(0, cursor);
        var match = before.match(/(^|\s)\/([a-z]*)$/i);
        if (match) {
            showSlashMenu(match[2] || '', el);
        } else {
            document.getElementById('slash-menu').classList.add('hidden');
        }
    }
    function showSlashMenu(query, _el) {
        var menu = document.getElementById('slash-menu');
        var filtered = query ? FALLBACK_SLASH_COMMANDS.filter(function(c) { return c.label.toLowerCase().includes(query); }) : FALLBACK_SLASH_COMMANDS;
        if (!filtered.length) { menu.classList.add('hidden'); return; }
        menu.innerHTML = '';
        filtered.forEach(function(cmd) {
            var btn = document.createElement('button');
            btn.type = 'button';
            btn.className = 'flex items-center gap-2 w-full px-3 py-1.5 text-xs text-foreground hover:bg-muted transition-colors';
            btn.innerHTML = cmd.icon + cmd.label;
            btn.onclick = function() { applySlashCommand(cmd); };
            menu.appendChild(btn);
        });
        menu.classList.remove('hidden');
    }

    /* ── Tiptap Init (loaded from local bundle) ── */
    (function() {
        var mountEl = document.getElementById('memo-editor');
        if (!mountEl) return;
        if (window.Tiptap) {
            var Editor = window.Tiptap.Editor;
            var StarterKit = window.Tiptap.StarterKit;
            var Placeholder = window.Tiptap.Placeholder;
            var Markdown = window.Tiptap.Markdown;
            var CodeBlockLowlight = window.Tiptap.CodeBlockLowlight;
            var lowlight = window.Tiptap.lowlight;
            var ImageExt = window.Tiptap.Image;
            var LinkExt = window.Tiptap.Link;

            mountEl.classList.remove('animate-pulse', 'bg-muted/30', 'rounded', 'shimmer-bg');
            // Remove parent's contenteditable so Tiptap's .ProseMirror handles editing
            mountEl.removeAttribute('contenteditable');
            mountEl.removeAttribute('data-empty');
            mountEl.oninput = null;
            mountEl.onkeydown = null;
            window.tiptapEditor = new Editor({
                element: mountEl,
                extensions: [
                    StarterKit.configure({ heading: { levels: [1, 2, 3] }, codeBlock: false }),
                    Placeholder.configure({ placeholder: "What's on your mind..." }),
                    Markdown,
                    CodeBlockLowlight.configure({ lowlight: lowlight }),
                    ImageExt,
                    LinkExt.configure({ openOnClick: false }),
                ],
                editorProps: {
                    attributes: { class: 'focus:outline-none text-base leading-relaxed' },
                    handleDrop: function(view, event, slice, moved) {
                        if (event.dataTransfer && event.dataTransfer.items && event.dataTransfer.items.length) {
                            var files = [];
                            for (var i = 0; i < event.dataTransfer.items.length; i++) {
                                if (event.dataTransfer.items[i].kind === 'file') {
                                    var f = event.dataTransfer.items[i].getAsFile();
                                    if (f) files.push(f);
                                }
                            }
                            if (files.length) {
                                uploadFilesForEditor(files);
                                event.preventDefault();
                                return true;
                            }
                        }
                        return false;
                    },
                    handlePaste: function(view, event) {
                        if (event.clipboardData && event.clipboardData.files && event.clipboardData.files.length) {
                            event.preventDefault();
                            event.stopPropagation();
                            uploadFilesForEditor(event.clipboardData.files);
                            return true;
                        }
                        return false;
                    },
                    handleKeyDown: function(view, event) {
                        if (event.key === 'ArrowDown') {
                            var state = view.state;
                            var $head = state.selection.$head;
                            for (var d = $head.depth; d >= 0; d--) {
                                var node = $head.node(d);
                                if (node && node.type.name === 'heading') {
                                    var afterPos = $head.after(d);
                                    if (afterPos >= state.doc.content.size) {
                                        event.preventDefault();
                                        var paragraph = state.schema.nodes.paragraph.create();
                                        var tr = state.tr.insert(afterPos, paragraph);
                                        var selClass = state.selection.constructor;
                                        var newResolved = tr.doc.resolve(afterPos + 1);
                                        tr.setSelection(selClass.near(newResolved));
                                        view.dispatch(tr);
                                        return true;
                                    }
                                    break;
                                }
                            }
                        }
                        return false;
                    }
                },
                onUpdate: function() {
                    var ed = window.tiptapEditor;
                    if (!ed) return;
                    var isEmpty = ed.isEmpty;
                    if (isEmpty) {
                        document.getElementById('memo-editor-input').value = '';
                    } else if (ed.storage.markdown) {
                        var md = ed.storage.markdown.getMarkdown();
                        document.getElementById('memo-editor-input').value = md;
                        isEmpty = md.trim() === '';
                    } else {
                        var html = ed.getHTML();
                        try {
                            var ts = new TurndownService({ headingStyle: 'atx' });
                            var md = ts.turndown(html);
                            document.getElementById('memo-editor-input').value = md;
                            isEmpty = md.trim() === '';
                        } catch(e) {
                            isEmpty = ed.getText().trim() === '';
                        }
                    }
                    updateSaveButtonState();
                },
            });
        } else {
            console.error('Tiptap bundle not loaded, using fallback editor');
            mountEl.classList.remove('animate-pulse', 'bg-muted/30', 'rounded', 'shimmer-bg');
            mountEl.setAttribute('contenteditable', 'true');
        }
    })();

    /* ── Keyboard Events ── */
    document.addEventListener('keydown', function(e) {
        var el = document.getElementById('memo-editor');
        if (!el || !document.activeElement || !el.contains(document.activeElement)) return;

        if ((e.ctrlKey || e.metaKey) && e.key === 'Enter') {
            e.preventDefault();
            document.getElementById('memo-editor-input').value = getTiptapMarkdown();
            var btn = document.getElementById('memo-form').querySelector('button[type="submit"]');
            if (btn) btn.click();
            return;
        }

        // Enter: let Tiptap handle it. Fallback handled via onFallbackKeydown.
        if (e.key === 'Enter' && window.tiptapEditor) {
            return;
        }

        // Slash menu only for Tiptap mode (fallback handled via onFallbackKeydown)
        if (e.key === '/' && window.tiptapEditor) {
            var ed = window.tiptapEditor;
            var cursorPos = ed.state.selection.$anchor.pos;
            var textBefore = ed.state.doc.textBetween(0, cursorPos, '\n', '');
            var match = textBefore.match(/(^|\s)\/([a-z]*)$/i);
            if (match) {
                setTimeout(function() { showSlashMenu(match[2] || '', el); }, 0);
            } else {
                document.getElementById('slash-menu').classList.add('hidden');
            }
            return;
        }
        if (e.key === 'Escape') {
            document.getElementById('slash-menu').classList.add('hidden');
        }
    });
    function setIconInactive(id) {
        var el = document.getElementById(id);
        if (el) el.className = 'p-2.5 rounded-xl text-muted-fg hover:bg-muted hover:text-foreground transition-colors';
    }
    function setIconActive(id) {
        document.getElementById(id).className = 'p-2.5 rounded-xl bg-blue-100 dark:bg-blue-900/30 text-blue-600 dark:text-blue-400 transition-colors';
    }
    function switchToTimeline() {
        var panel = document.getElementById('sidebar-panel');
        panel.classList.remove('hidden');
        panel.style.display = 'flex';
        document.getElementById('resources-panel').classList.add('hidden');
        document.getElementById('resources-panel').style.display = 'none';
        document.getElementById('notes-panel').classList.add('hidden');
        document.getElementById('notes-panel').style.display = 'none';
        document.getElementById('timeline-view').classList.remove('hidden');
        document.getElementById('timeline-view').classList.add('flex');
        document.getElementById('note-detail-view').classList.add('hidden');
        setIconActive('icon-timeline');
        setIconInactive('icon-notes');
        setIconInactive('icon-resources');
        htmx.ajax('GET', '/sidebar-timeline', {target: '#sidebar-content', swap: 'innerHTML'});
    }
    function switchToNotes() {
        document.getElementById('sidebar-panel').classList.add('hidden');
        document.getElementById('sidebar-panel').style.display = 'none';
        document.getElementById('resources-panel').classList.add('hidden');
        document.getElementById('resources-panel').style.display = 'none';
        document.getElementById('notes-panel').classList.remove('hidden');
        document.getElementById('notes-panel').style.display = 'flex';
        setIconActive('icon-notes');
        setIconInactive('icon-timeline');
        setIconInactive('icon-resources');
        htmx.ajax('GET', '/notes-panel', {target: '#notes-panel', swap: 'innerHTML'});
    }
    function openNote(noteId) {
        document.getElementById('timeline-view').classList.add('hidden');
        document.getElementById('timeline-view').classList.remove('flex');
        document.getElementById('note-detail-view').classList.remove('hidden');
        document.getElementById('note-detail-view').innerHTML = '<div class="flex items-center justify-center h-full"><div class="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600"></div></div>';
        htmx.ajax('GET', '/note/' + noteId, {target: '#note-detail-view', swap: 'innerHTML'});
    }
    function editMemo(id) {
        var card = document.getElementById('memo-' + id);
        if (!card) return;
        card.querySelector('.memo-display').classList.add('hidden');
        card.querySelector('.memo-edit').classList.remove('hidden');
    }
    function cancelEdit(id) {
        var card = document.getElementById('memo-' + id);
        if (!card) return;
        card.querySelector('.memo-display').classList.remove('hidden');
        card.querySelector('.memo-edit').classList.add('hidden');
    }
    function deleteMemo(id) {
        if (!confirm('Delete this memo?')) return;
        var card = document.getElementById('memo-' + id);
        if (!card) return;
        fetch('/memos/' + id, { method: 'DELETE' }).then(function(r) {
            if (r.ok) {
                card.remove();
                htmx.trigger('body', 'memoUpdated');
            }
        });
    }
    function visIcon(value) {
        if (value === 'public') return '<svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3.055 11H5a2 2 0 012 2v1a2 2 0 002 2 2 2 0 012 2v2.945M8 3.935V5.5A2.5 2.5 0 0010.5 8h.5a2 2 0 012 2 2 2 0 104 0 2 2 0 012-2h1.064M15 20.488V18a2 2 0 012-2h3.064M21 12a9 9 0 11-18 0 9 9 0 0118 0z"/></svg>';
        if (value === 'protected') return '<svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><rect x="3" y="11" width="18" height="11" rx="2" stroke-width="2"/><path d="M7 11V7a5 5 0 0110 0v4" stroke-width="2"/></svg>';
        return '<svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z"/></svg>';
    }
    function visLabel(value) {
        return value.charAt(0).toUpperCase() + value.slice(1);
    }
    function toggleVisDropdown(btn) {
        var menu = btn.parentElement.querySelector('.vis-dropdown-menu');
        var open = !menu.classList.contains('hidden');
        document.querySelectorAll('.vis-dropdown-menu').forEach(function(m) { m.classList.add('hidden'); });
        if (!open) menu.classList.remove('hidden');
    }
    function selectVis(btn) {
        var value = btn.getAttribute('data-vis-value');
        var dropdown = btn.closest('.visibility-dropdown');
        dropdown.querySelector('input[name="visibility"]').value = value;
        dropdown.setAttribute('data-vis', value);
        dropdown.querySelector('.vis-label').innerHTML = visIcon(value) + visLabel(value);
        dropdown.querySelector('.vis-dropdown-menu').classList.add('hidden');
    }
    document.addEventListener('click', function(e) {
        if (!e.target.closest('.visibility-dropdown')) {
            document.querySelectorAll('.vis-dropdown-menu').forEach(function(m) { m.classList.add('hidden'); });
        }
        if (!e.target.closest('#emoji-picker') && !e.target.closest('[onclick*=\"toggleEmojiPicker\"]')) {
            document.getElementById('emoji-picker').classList.add('hidden');
        }
        if (!e.target.closest('#plus-menu') && !e.target.closest('#link-memo-dropdown') && !e.target.closest('[onclick*=\"togglePlusMenu\"]') && !e.target.closest('[onclick*=\"toggleLinkMemo\"]')) {
            document.getElementById('plus-menu').classList.add('hidden');
            document.getElementById('link-memo-dropdown').classList.add('hidden');
        }
    });
    function uploadFiles(files) {
        if (!files.length) return;
        var formData = new FormData();
        for (var i = 0; i < files.length; i++) formData.append('file', files[i]);
        fetch('/resources', { method: 'POST', body: formData }).then(function(r) { return r.json(); }).then(function(data) {
            if (data.resources && data.resources.length) {
                var combined = data.resources.map(function(r) { return r.markdown; }).join('\n\n');
                insertContenteditable(combined);
                refreshResourcesPanel();
            }
        }).catch(function(err) { console.error('Resource upload failed:', err); });
    }
    var editorAttachments = [];
    function formatBytes(bytes) {
        if (!bytes) return '0 B';
        var k = 1024;
        var sizes = ['B', 'KB', 'MB', 'GB'];
        var i = Math.floor(Math.log(bytes) / Math.log(k));
        return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
    }
    function renderEditorAttachments() {
        var container = document.getElementById('attachment-preview-container');
        var list = document.getElementById('attachment-preview-list');
        if (!list || !container) return;
        
        list.innerHTML = '';
        if (editorAttachments.length === 0) {
            container.classList.add('hidden');
            return;
        }
        container.classList.remove('hidden');
        
        var header = document.createElement('div');
        header.className = 'text-xs font-semibold text-muted-fg px-4 py-2 border-b border-border bg-muted/20 flex items-center justify-between';
        header.innerHTML = '<span>Attachments (' + editorAttachments.length + ')</span>';
        list.appendChild(header);
        
        editorAttachments.forEach(function(r, idx) {
            var isImg = r.markdown.startsWith('!');
            var row = document.createElement('div');
            row.className = 'flex items-center justify-between gap-3 px-4 py-2 border-b border-border last:border-b-0 hover:bg-muted/10 transition-colors';
            
            var thumb = document.createElement('div');
            thumb.className = 'w-10 h-10 rounded border border-border overflow-hidden shrink-0 flex items-center justify-center bg-muted';
            if (isImg) {
                thumb.innerHTML = '<img src="/resources/' + r.id + '" class="w-full h-full object-cover">';
            } else {
                thumb.innerHTML = '<svg class="w-5 h-5 text-muted-fg" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"/></svg>';
            }
            row.appendChild(thumb);
            
            var details = document.createElement('div');
            details.className = 'flex-1 min-w-0';
            var sizeStr = formatBytes(r.size);
            details.innerHTML = '<div class="text-xs font-medium truncate text-foreground" title="' + r.name + '">' + r.name + '</div>' +
                                '<div class="text-[10px] text-muted-fg">' + sizeStr + '</div>';
            row.appendChild(details);
            
            var actions = document.createElement('div');
            actions.className = 'flex items-center gap-1 shrink-0';
            
            var upBtn = document.createElement('button');
            upBtn.type = 'button';
            upBtn.className = 'p-1 hover:bg-muted rounded text-muted-fg hover:text-foreground transition-colors disabled:opacity-30 disabled:hover:bg-transparent';
            upBtn.disabled = idx === 0;
            upBtn.title = 'Move Up';
            upBtn.innerHTML = '<svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 15l7-7 7 7"/></svg>';
            upBtn.onclick = function() { moveAttachment(idx, -1); };
            actions.appendChild(upBtn);
            
            var downBtn = document.createElement('button');
            downBtn.type = 'button';
            downBtn.className = 'p-1 hover:bg-muted rounded text-muted-fg hover:text-foreground transition-colors disabled:opacity-30 disabled:hover:bg-transparent';
            downBtn.disabled = idx === editorAttachments.length - 1;
            downBtn.title = 'Move Down';
            downBtn.innerHTML = '<svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7"/></svg>';
            downBtn.onclick = function() { moveAttachment(idx, 1); };
            actions.appendChild(downBtn);
            
            var delBtn = document.createElement('button');
            delBtn.type = 'button';
            delBtn.className = 'p-1 hover:bg-red-500/10 hover:text-red-500 rounded text-muted-fg transition-colors';
            delBtn.title = 'Delete Attachment';
            delBtn.innerHTML = '<svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"/></svg>';
            delBtn.onclick = function() { deleteAttachment(idx); };
            actions.appendChild(delBtn);
            
            row.appendChild(actions);
            list.appendChild(row);
        });
    }
    function swapMarkdownInEditor(md1, md2) {
        var ed = window.tiptapEditor;
        var currentText = "";
        if (ed) {
            currentText = ed.storage.markdown.getMarkdown();
        } else {
            var el = document.getElementById('memo-editor');
            currentText = el ? (el.innerText || el.textContent || '') : '';
        }

        var idx1 = currentText.indexOf(md1);
        var idx2 = currentText.indexOf(md2);
        if (idx1 !== -1 && idx2 !== -1) {
            var newText = "";
            if (idx1 < idx2) {
                newText = currentText.substring(0, idx1) + 
                          md2 + 
                          currentText.substring(idx1 + md1.length, idx2) + 
                          md1 + 
                          currentText.substring(idx2 + md2.length);
            } else {
                newText = currentText.substring(0, idx2) + 
                          md1 + 
                          currentText.substring(idx2 + md2.length, idx1) + 
                          md2 + 
                          currentText.substring(idx1 + md1.length);
            }
            if (ed) {
                ed.commands.setContent(newText);
            } else {
                var el = document.getElementById('memo-editor');
                if (el) {
                    el.innerText = newText;
                    document.getElementById('memo-editor-input').value = newText;
                }
            }
        }
    }
    function moveAttachment(index, direction) {
        var newIndex = index + direction;
        if (newIndex < 0 || newIndex >= editorAttachments.length) return;
        
        var att1 = editorAttachments[index];
        var att2 = editorAttachments[newIndex];
        
        editorAttachments[index] = att2;
        editorAttachments[newIndex] = att1;
        
        swapMarkdownInEditor(att1.markdown, att2.markdown);
        renderEditorAttachments();
    }
    function escapeRegExp(string) {
        return string.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
    }
    function deleteAttachment(index) {
        var att = editorAttachments[index];
        editorAttachments.splice(index, 1);
        
        var pattern = new RegExp('!?\\[' + escapeRegExp(att.name) + '\\]\\(/resources/' + att.id + '\\)', 'g');
        var patternFallback = new RegExp('!?\\[.*?\\]\\(/resources/' + att.id + '\\)', 'g');
        
        var ed = window.tiptapEditor;
        if (ed) {
            var currentMarkdown = ed.storage.markdown.getMarkdown();
            var newMarkdown = currentMarkdown.replace(pattern, '').replace(patternFallback, '');
            ed.commands.setContent(newMarkdown);
        } else {
            var el = document.getElementById('memo-editor');
            if (el) {
                var currentText = el.innerText || el.textContent || '';
                var newText = currentText.replace(pattern, '').replace(patternFallback, '');
                el.innerText = newText;
                document.getElementById('memo-editor-input').value = newText;
            }
        }
        
        renderEditorAttachments();
        
        fetch('/resources/' + att.id, { method: 'DELETE' }).then(function(r) {
            if (r.ok) {
                var panel = document.getElementById('resources-panel');
                if (panel && !panel.classList.contains('hidden')) refreshResourcesPanel();
                htmx.trigger('body', 'memoUpdated');
            }
        });
    }
    function uploadFilesForEditor(files) {
        if (!files.length) return;
        var formData = new FormData();
        for (var i = 0; i < files.length; i++) formData.append('file', files[i]);
        fetch('/resources', { method: 'POST', body: formData }).then(function(r) { return r.json(); }).then(function(data) {
            if (data.resources && data.resources.length) {
                var combined = data.resources.map(function(r) { return r.markdown; }).join('\n\n');
                insertContenteditable(combined);
                data.resources.forEach(function(r) {
                    editorAttachments.push({
                        id: r.id,
                        name: r.original_name,
                        size: r.size,
                        markdown: r.markdown
                    });
                });
                renderEditorAttachments();
                var panel = document.getElementById('resources-panel');
                if (panel && !panel.classList.contains('hidden')) refreshResourcesPanel();
            }
        }).catch(function(err) { console.error('Resource upload failed:', err); });
    }
    function handleDrop(e) {
        e.preventDefault();
        e.stopPropagation();
        var form = document.getElementById('memo-form');
        if (form) form.classList.remove('border-blue-500');
        var files = [];
        var seen = new Set();
        function add(f) {
            if (!f) return;
            var key = f.name + '|' + f.size + '|' + f.type;
            if (!seen.has(key)) {
                seen.add(key);
                files.push(f);
            }
        }
        if (e.dataTransfer.files && e.dataTransfer.files.length) {
            for (var i = 0; i < e.dataTransfer.files.length; i++) add(e.dataTransfer.files[i]);
        }
        if (e.dataTransfer.items && e.dataTransfer.items.length) {
            for (var i = 0; i < e.dataTransfer.items.length; i++) {
                if (e.dataTransfer.items[i].kind === 'file') add(e.dataTransfer.items[i].getAsFile());
            }
        }
        if (files.length) uploadFilesForEditor(files);
    }
    document.addEventListener('paste', function(e) {
        var el = document.getElementById('memo-editor');
        if (!el || !document.activeElement || !el.contains(document.activeElement)) return;
        if (window.tiptapEditor) return;
        if (e.clipboardData && e.clipboardData.files && e.clipboardData.files.length) {
            e.preventDefault();
            uploadFilesForEditor(e.clipboardData.files);
        }
    });
    function refreshResourcesPanel() {
        var panel = document.getElementById('resources-panel');
        if (panel && !panel.classList.contains('hidden')) {
            htmx.ajax('GET', '/resources-feed', {target: '#resources-panel', swap: 'innerHTML'});
        }
    }
    function switchToResources() {
        document.getElementById('sidebar-panel').classList.add('hidden');
        document.getElementById('sidebar-panel').style.display = 'none';
        document.getElementById('notes-panel').classList.add('hidden');
        document.getElementById('notes-panel').style.display = 'none';
        document.getElementById('resources-panel').classList.remove('hidden');
        document.getElementById('resources-panel').style.display = 'flex';
        setIconInactive('icon-timeline');
        setIconInactive('icon-notes');
        setIconActive('icon-resources');
        refreshResourcesPanel();
    }
    function updateBulkActions() {
        var checkboxes = document.querySelectorAll('.res-checkbox');
        var checked = Array.from(checkboxes).filter(function(c) { return c.checked; });
        var bar = document.getElementById('bulk-actions');
        if (!bar) return;
        if (checked.length) {
            bar.classList.remove('hidden');
            document.getElementById('selected-count').textContent = checked.length;
        } else {
            bar.classList.add('hidden');
            document.getElementById('select-all').checked = false;
        }
    }
    function toggleSelectAll() {
        var checked = document.getElementById('select-all').checked;
        document.querySelectorAll('.res-checkbox').forEach(function(c) { c.checked = checked; });
        updateBulkActions();
    }
    function deleteSelectedResources() {
        var checkboxes = document.querySelectorAll('.res-checkbox:checked');
        var ids = Array.from(checkboxes).map(function(c) { return parseInt(c.value); });
        if (!ids.length) return;
        if (!confirm('Delete ' + ids.length + ' selected resource' + (ids.length > 1 ? 's' : '') + '?')) return;
        fetch('/resources/bulk-delete', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ ids: ids })
        }).then(function(r) { return r.json(); }).then(function(data) {
            if (data.deleted > 0) {
                refreshResourcesPanel();
                htmx.trigger('body', 'memoUpdated');
            }
        });
    }
    function navigateFromHash() {
        var hash = location.hash || '#timeline';
        if (hash === '#notes') switchToNotes();
        else if (hash === '#resources') switchToResources();
        else switchToTimeline();
    }
    window.addEventListener('hashchange', navigateFromHash);
    document.addEventListener('DOMContentLoaded', navigateFromHash);
    function openImageModal(src) {
        document.getElementById('image-modal').classList.remove('hidden');
        document.getElementById('image-modal').querySelector('img').src = src;
    }
    function closeImageModal() {
        document.getElementById('image-modal').classList.add('hidden');
    }
</script>
{% endblock %}"##;

const NOTES_PANEL_TEMPLATE: &str = r#"<div class="flex flex-col h-full">
    <div class="px-4 py-3 border-b border-border flex-shrink-0">
        <h2 class="text-xs font-semibold text-muted-fg uppercase tracking-wider">Notes</h2>
    </div>
    <div class="flex-1 overflow-y-auto p-2 space-y-1">
        {% if notes %}
            {% for note in notes %}
            <div onclick="openNote({{ note.id }})"
                class="p-3 rounded-lg hover:bg-muted cursor-pointer transition-colors flex gap-3 items-start justify-between border-b border-border/30 last:border-0">
                <div class="flex-1 min-w-0">
                    <p class="text-sm font-medium text-foreground truncate flex items-center gap-1.5">
                        {{ note.title }}
                        {% if note.visibility == 'public' %}
                        <svg class="w-3.5 h-3.5 text-green-600 dark:text-green-400 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3.055 11H5a2 2 0 012 2v1a2 2 0 002 2 2 2 0 012 2v2.945M8 3.935V5.5A2.5 2.5 0 0010.5 8h.5a2 2 0 012 2 2 2 0 104 0 2 2 0 012-2h1.064M15 20.488V18a2 2 0 012-2h3.064M21 12a9 9 0 11-18 0 9 9 0 0118 0z"/></svg>
                        {% elif note.visibility == 'protected' %}
                        <svg class="w-3.5 h-3.5 text-amber-600 dark:text-amber-400 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24"><rect x="3" y="11" width="18" height="11" rx="2" stroke-width="2"/><path d="M7 11V7a5 5 0 0110 0v4" stroke-width="2"/></svg>
                        {% endif %}
                    </p>
                    <p class="text-[10px] text-muted-fg mt-0.5">{{ note.created_at }}</p>
                    {% if note.tags %}
                    <div class="flex flex-wrap gap-1 mt-1.5">
                        {% for tag in note.tags %}
                        <span class="inline-block px-1.5 py-0.5 text-[9px] font-medium rounded bg-blue-50 dark:bg-blue-900/20 text-blue-600 dark:text-blue-400">
                            #{{ tag }}
                        </span>
                        {% endfor %}
                    </div>
                    {% endif %}
                </div>
                {% if note.first_image_id %}
                <div class="w-12 h-12 rounded-lg overflow-hidden border border-border shrink-0 bg-[#f0f0eb] dark:bg-[#3e4045]">
                    <img src="/resources/{{ note.first_image_id }}" class="w-full h-full object-cover" loading="lazy">
                </div>
                {% endif %}
            </div>
            {% endfor %}
        {% else %}
            <p class="text-sm text-gray-400 p-3 text-center">No notes yet</p>
        {% endif %}
    </div>
</div>"#;

const NOTE_DETAIL_TEMPLATE: &str = r#"<div>
    <button onclick="switchToTimeline()"
        class="flex items-center gap-1.5 text-sm text-muted-fg hover:text-gray-700 dark:hover:text-gray-200 mb-4 transition-colors">
        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7"/>
        </svg>
        Back to timeline
    </button>
    <div class="memo-content">{{ content_html|safe }}</div>
    
    {% if resources and resources|length > 0 %}
    <div class="mt-3 border border-border rounded-xl overflow-hidden bg-muted/20">
        <div class="flex items-center gap-1.5 px-3 py-1.5 border-b border-border bg-muted/30 text-[10px] font-semibold text-muted-fg uppercase tracking-wider">
            <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15.172 7l-6.586 6.586a2 2 0 102.828 2.828l6.414-6.586a4 4 0 00-5.656-5.656l-6.415 6.585a6 6 0 108.486 8.486L20.5 13"/></svg>
            Attachments ({{ resources|length }})
        </div>
        <div class="p-3 bg-card space-y-3">
            {% for res in resources %}
                {% if res.is_image %}
                <div class="rounded-lg overflow-hidden border border-border bg-muted/10 flex items-center justify-center" style="max-height:calc((100vw - 18rem)/4)">
                    <img src="/resources/{{ res.id }}" class="max-w-full object-contain cursor-zoom-in" style="max-height:calc((100vw - 18rem)/4)" loading="lazy" onclick="openImageModal(this.src)">
                </div>
                {% else %}
                <div class="flex items-center gap-2 p-2 rounded-lg border border-border hover:bg-muted/40 transition-colors">
                    <div class="w-8 h-8 rounded border border-border overflow-hidden shrink-0 bg-muted flex items-center justify-center">
                        <svg class="w-4 h-4 text-muted-fg" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"/></svg>
                    </div>
                    <div class="min-w-0 flex-1">
                        <p class="font-medium truncate text-foreground text-[13px]">{{ res.original_name }}</p>
                        <p class="text-[10px] text-muted-fg">{{ res.mime_type }} · {{ res.size }}</p>
                    </div>
                    <a href="/resources/{{ res.id }}" download class="p-1 rounded text-muted-fg hover:text-foreground hover:bg-muted transition-colors shrink-0" title="Download">
                        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4"/></svg>
                    </a>
                </div>
                {% endif %}
            {% endfor %}
        </div>
    </div>
    {% endif %}
    
    <p class="text-xs text-gray-400 mt-4 pt-3 border-t border-border">{{ created_at }}</p>
</div>"#;

const RESOURCES_PANEL_TEMPLATE: &str = r##"<div class="flex flex-col h-full">
    <div class="px-4 py-3 border-b border-border flex-shrink-0">
        <h2 class="text-xs font-semibold text-muted-fg uppercase tracking-wider">Resources</h2>
    </div>
    <div class="px-3 py-2 border-b border-border flex-shrink-0">
        <div class="relative">
            <input type="file" multiple id="file-input" class="hidden" onchange="uploadFiles(this.files)">
            <button onclick="document.getElementById('file-input').click()"
                class="w-full px-3 py-2 text-xs font-medium text-blue-600 dark:text-blue-400 bg-blue-50 dark:bg-blue-900/20 hover:bg-blue-100 dark:hover:bg-blue-900/30 rounded-lg transition-colors text-center">
                Upload Files
            </button>
        </div>
    </div>
    <div id="bulk-actions" class="hidden px-3 py-1.5 border-b border-border flex-shrink-0 flex items-center justify-between bg-muted/20">
        <label class="flex items-center gap-1.5 text-xs text-muted-fg cursor-pointer">
            <input type="checkbox" id="select-all" onchange="toggleSelectAll()" class="rounded border-border">
            Select All
        </label>
        <button onclick="deleteSelectedResources()" class="px-2 py-1 text-xs font-medium text-red-500 hover:bg-red-50 dark:hover:bg-red-900/20 rounded-md transition-colors">
            Delete Selected (<span id="selected-count">0</span>)
        </button>
    </div>
    <div class="flex-1 overflow-y-auto p-2 space-y-1" id="resources-list">
        {% if resources %}
            <p class="text-[10px] text-muted-fg px-2 pb-1.5 border-b border-border/30 mb-1">Click a resource to add it to your note.</p>
            {% for res in resources %}
            <div class="flex items-center gap-1.5 p-2 rounded-lg hover:bg-muted transition-colors group/res">
                <input type="checkbox" class="res-checkbox rounded border-border/60" value="{{ res.id }}" onchange="updateBulkActions()">
                {% if res.is_image %}
                <div class="w-10 h-10 rounded-lg overflow-hidden flex-shrink-0 bg-[#f0f0eb] dark:bg-[#3e4045]">
                    <img src="/resources/{{ res.id }}" class="w-full h-full object-cover" loading="lazy">
                </div>
                {% else %}
                <div class="w-10 h-10 rounded-lg flex-shrink-0 bg-[#f0f0eb] dark:bg-[#3e4045] flex items-center justify-center">
                    <svg class="w-5 h-5 text-[#8e8e8a]" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 21h10a2 2 0 002-2V9.414a1 1 0 00-.293-.707l-5.414-5.414A1 1 0 0012.586 3H7a2 2 0 00-2 2v14a2 2 0 002 2z"/>
                    </svg>
                </div>
                {% endif %}
                <div class="flex-1 min-w-0 cursor-pointer" onclick="insertContenteditable('{% if res.is_image %}![{{ res.original_name }}](/resources/{{ res.id }}){% else %}[{{ res.original_name }}](/resources/{{ res.id }}){% endif %}')">
                    <p class="text-xs font-medium text-foreground truncate">{{ res.original_name }}</p>
                    <p class="text-[10px] text-[#8e8e8a]">{{ res.size_str }}</p>
                </div>
                <button onclick="if(confirm('Delete this resource?')){var e=this;fetch('/resources/{{ res.id }}',{method:'DELETE'}).then(function(r){if(r.ok){e.closest('.group\\/res').remove();refreshResourcesPanel();htmx.trigger('body','memoUpdated')}})}"
                    class="p-1 rounded-md text-[#8e8e8a] hover:text-red-500 hover:bg-red-50 dark:hover:bg-red-900/20 transition-all" title="Delete">
                    <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"/>
                    </svg>
                </button>
            </div>
            {% endfor %}
        {% else %}
            <p class="text-xs text-muted-fg text-center py-10">No resources yet.<br>Drag & drop files into the editor or click Upload.</p>
        {% endif %}
    </div>
</div>"##;

const MEMO_FRAGMENT: &str = r##"<div id="memo-{{ id }}" class="p-4 bg-card rounded-xl border border-border shadow-sm hover:shadow-md transition-shadow group/memo">
    <div class="memo-display">
        <div class="flex items-center gap-2 mb-2">
            <div class="flex items-center gap-1.5 min-w-0">
                <span class="text-sm font-medium text-foreground truncate">{{ username|default("") }}</span>
                <span class="text-xs text-muted-fg whitespace-nowrap">· <span class="relative-time" data-time="{{ created_at }}">{{ created_at_relative|default(created_at) }}</span></span>
                {% if visibility == 'public' %}
                <span class="text-xs text-green-600 dark:text-green-400 select-none" title="Public">
                    <svg class="w-3.5 h-3.5 inline" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3.055 11H5a2 2 0 012 2v1a2 2 0 002 2 2 2 0 012 2v2.945M8 3.935V5.5A2.5 2.5 0 0010.5 8h.5a2 2 0 012 2 2 2 0 104 0 2 2 0 012-2h1.064M15 20.488V18a2 2 0 012-2h3.064M21 12a9 9 0 11-18 0 9 9 0 0118 0z"/></svg>
                </span>
                {% elif visibility == 'protected' %}
                <span class="text-xs text-amber-600 dark:text-amber-400 select-none" title="Protected">
                    <svg class="w-3.5 h-3.5 inline" fill="none" stroke="currentColor" viewBox="0 0 24 24"><rect x="3" y="11" width="18" height="11" rx="2" stroke-width="2"/><path d="M7 11V7a5 5 0 0110 0v4" stroke-width="2"/></svg>
                </span>
                {% endif %}
            </div>
            <div class="ml-auto flex items-center gap-1 opacity-0 group-hover/memo:opacity-100 transition-opacity">
                <button onclick="editMemo({{ id }})"
                    class="p-1 rounded-md text-muted-fg hover:text-foreground hover:bg-muted transition-colors" title="Edit">
                    <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z"/>
                    </svg>
                </button>
                <button onclick="deleteMemo({{ id }})"
                    class="p-1 rounded-md text-[#8e8e8a] hover:text-red-500 hover:bg-red-50 dark:hover:bg-red-900/20 transition-colors" title="Delete">
                    <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"/>
                    </svg>
                </button>
            </div>
        </div>
        <div class="memo-content text-foreground text-[15px] leading-relaxed">{{ content_html|safe }}</div>
        
        {% if tags and tags|length > 0 %}
        <div class="flex flex-wrap gap-1 mt-2">
            {% for tag in tags %}
            <span class="inline-block px-1.5 py-0.5 text-[9px] font-medium rounded bg-blue-50 dark:bg-blue-900/20 text-blue-600 dark:text-blue-400">
                #{{ tag }}
            </span>
            {% endfor %}
        </div>
        {% endif %}

        {% if resources and resources|length > 0 %}
        <div class="mt-3 border border-border rounded-xl overflow-hidden bg-muted/20">
            <div class="flex items-center gap-1.5 px-3 py-1.5 border-b border-border bg-muted/30 text-[10px] font-semibold text-muted-fg uppercase tracking-wider">
                <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15.172 7l-6.586 6.586a2 2 0 102.828 2.828l6.414-6.586a4 4 0 00-5.656-5.656l-6.415 6.585a6 6 0 108.486 8.486L20.5 13"/></svg>
                Attachments ({{ resources|length }})
            </div>
            <div class="p-3 bg-card space-y-3">
                {% for res in resources %}
                    {% if res.is_image %}
                    <div class="rounded-lg overflow-hidden border border-border bg-muted/10 flex items-center justify-center" style="max-height:calc((100vw - 18rem)/4)">
                        <img src="/resources/{{ res.id }}" class="max-w-full object-contain cursor-zoom-in" style="max-height:calc((100vw - 18rem)/4)" loading="lazy" onclick="openImageModal(this.src)">
                    </div>
                    {% else %}
                    <div class="flex items-center gap-2 p-2 rounded-lg border border-border hover:bg-muted/40 transition-colors">
                        <div class="w-8 h-8 rounded border border-border overflow-hidden shrink-0 bg-muted flex items-center justify-center">
                            <svg class="w-4 h-4 text-muted-fg" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"/></svg>
                            </div>
                        <div class="min-w-0 flex-1">
                            <p class="font-medium truncate text-foreground text-[13px]">{{ res.original_name }}</p>
                            <p class="text-[10px] text-muted-fg">{{ res.mime_type }} · {{ res.size }}</p>
                        </div>
                        <a href="/resources/{{ res.id }}" download class="p-1 rounded text-muted-fg hover:text-foreground hover:bg-muted transition-colors shrink-0" title="Download">
                            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4"/></svg>
                        </a>
                    </div>
                    {% endif %}
                {% endfor %}
            </div>
        </div>
        {% endif %}
    </div>
    <div class="memo-edit hidden">
        <form hx-put="/memos/{{ id }}" hx-target="#memo-{{ id }}" hx-swap="outerHTML" hx-on::after-request="if(event.detail.successful){htmx.trigger('body','memoUpdated')}">
            <textarea name="content" rows="3" required
                class="w-full px-3 py-2 bg-white dark:bg-background border border-border rounded-lg text-sm text-foreground placeholder-muted-fg focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent resize-none mb-2"
            >{{ content }}</textarea>
            <div class="flex items-center justify-between">
                <div class="flex items-center gap-1">
                    {% set vis = visibility|default("private") %}
                    <div class="visibility-dropdown relative" data-vis="{{ vis }}">
                        <button type="button" onclick="toggleVisDropdown(this)" class="flex items-center gap-1 px-1.5 py-1 rounded-md text-muted-fg hover:text-foreground hover:bg-muted transition-colors text-xs">
                            <span class="vis-label flex items-center gap-1">
                                {% if vis == 'public' %}<svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3.055 11H5a2 2 0 012 2v1a2 2 0 002 2 2 2 0 012 2v2.945M8 3.935V5.5A2.5 2.5 0 0010.5 8h.5a2 2 0 012 2 2 2 0 104 0 2 2 0 012-2h1.064M15 20.488V18a2 2 0 012-2h3.064M21 12a9 9 0 11-18 0 9 9 0 0118 0z"/></svg>Public
                                {% elif vis == 'protected' %}<svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><rect x="3" y="11" width="18" height="11" rx="2" stroke-width="2"/><path d="M7 11V7a5 5 0 0110 0v4" stroke-width="2"/></svg>Protected
                                {% else %}<svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z"/></svg>Private{% endif %}
                            </span>
                            <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7"/></svg>
                        </button>
                        <div class="vis-dropdown-menu hidden absolute top-full left-0 mt-1 bg-card border border-border rounded-lg shadow-lg py-1 min-w-[140px] z-50">
                            <button type="button" data-vis-value="public" onclick="selectVis(this)" class="flex items-center gap-2 w-full px-3 py-1.5 text-xs text-foreground hover:bg-muted transition-colors">
                                <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3.055 11H5a2 2 0 012 2v1a2 2 0 002 2 2 2 0 012 2v2.945M8 3.935V5.5A2.5 2.5 0 0010.5 8h.5a2 2 0 012 2 2 2 0 104 0 2 2 0 012-2h1.064M15 20.488V18a2 2 0 012-2h3.064M21 12a9 9 0 11-18 0 9 9 0 0118 0z"/></svg>
                                Public
                            </button>
                            <button type="button" data-vis-value="protected" onclick="selectVis(this)" class="flex items-center gap-2 w-full px-3 py-1.5 text-xs text-foreground hover:bg-muted transition-colors">
                                <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><rect x="3" y="11" width="18" height="11" rx="2" stroke-width="2"/><path d="M7 11V7a5 5 0 0110 0v4" stroke-width="2"/></svg>
                                Protected
                            </button>
                            <button type="button" data-vis-value="private" onclick="selectVis(this)" class="flex items-center gap-2 w-full px-3 py-1.5 text-xs text-foreground hover:bg-muted transition-colors">
                                <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z"/></svg>
                                Private
                            </button>
                        </div>
                        <input type="hidden" name="visibility" value="{{ vis }}">
                    </div>
                </div>
                <div class="flex items-center gap-1.5">
                    <button type="button" onclick="cancelEdit({{ id }})"
                        class="px-3 py-1.5 text-xs font-medium text-muted-fg">Cancel</button>
                    <button type="submit"
                        class="px-3 py-1.5 text-xs font-medium bg-blue-600 hover:bg-blue-700 text-white rounded-lg transition-colors focus:outline-none focus:ring-2 focus:ring-blue-500">Save</button>
                </div>
            </div>
        </form>
    </div>
</div>"##;

const MEMO_EDIT_FORM: &str = r##"<form hx-put="/memos/{{ id }}" hx-target="#memo-{{ id }}" hx-swap="outerHTML" hx-on::after-request="if(event.detail.successful){htmx.trigger('body','memoUpdated')}">
    <textarea name="content" rows="3" required
        class="w-full px-3 py-2 bg-white dark:bg-background border border-border rounded-lg text-sm text-foreground placeholder-muted-fg focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent resize-none mb-2"
    >{{ content }}</textarea>
    <div class="flex items-center justify-between">
        <div class="visibility-dropdown relative" data-vis="private">
            <button type="button" onclick="toggleVisDropdown(this)" class="flex items-center gap-1 px-1.5 py-1 rounded-md text-muted-fg hover:text-foreground hover:bg-muted transition-colors text-xs">
                <span class="vis-label flex items-center gap-1"><svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z"/></svg>Private</span>
                <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7"/></svg>
            </button>
            <div class="vis-dropdown-menu hidden absolute top-full left-0 mt-1 bg-card border border-border rounded-lg shadow-lg py-1 min-w-[140px] z-50">
                <button type="button" data-vis-value="public" onclick="selectVis(this)" class="flex items-center gap-2 w-full px-3 py-1.5 text-xs text-foreground hover:bg-muted transition-colors">
                    <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3.055 11H5a2 2 0 012 2v1a2 2 0 002 2 2 2 0 012 2v2.945M8 3.935V5.5A2.5 2.5 0 0010.5 8h.5a2 2 0 012 2 2 2 0 104 0 2 2 0 012-2h1.064M15 20.488V18a2 2 0 012-2h3.064M21 12a9 9 0 11-18 0 9 9 0 0118 0z"/></svg>
                    Public
                </button>
                <button type="button" data-vis-value="protected" onclick="selectVis(this)" class="flex items-center gap-2 w-full px-3 py-1.5 text-xs text-foreground hover:bg-muted transition-colors">
                    <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><rect x="3" y="11" width="18" height="11" rx="2" stroke-width="2"/><path d="M7 11V7a5 5 0 0110 0v4" stroke-width="2"/></svg>
                    Protected
                </button>
                <button type="button" data-vis-value="private" onclick="selectVis(this)" class="flex items-center gap-2 w-full px-3 py-1.5 text-xs text-foreground hover:bg-muted transition-colors">
                    <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z"/></svg>
                    Private
                </button>
            </div>
            <input type="hidden" name="visibility" value="private">
        </div>
        <div class="flex items-center gap-1.5">
            <button type="submit" class="px-3 py-1.5 text-xs font-medium bg-blue-600 hover:bg-blue-700 text-white rounded-lg transition-colors">Save</button>
        </div>
    </div>
</form>"##;

const SIDEBAR_TIMELINE_TEMPLATE: &str = r##"<div class="flex flex-col h-full">
    <!-- Search -->
    <div class="px-3 pt-3 pb-2 flex-shrink-0">
        <input type="text" name="q" placeholder="Search memos..."
            hx-get="/search"
            hx-target="#timeline"
            hx-swap="innerHTML"
            hx-trigger="keyup changed delay:400ms, search"
            hx-on::before-request="if (this.value === '') { event.detail.pathInfo.requestPath = '/memos-feed' }"
            class="w-full px-3 py-1.5 bg-card border border-border rounded-lg text-sm text-foreground placeholder-muted-fg focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent transition-all" />
    </div>

    <!-- Calendar -->
    <div class="px-3 py-2 flex-shrink-0">
        <div class="flex items-center justify-between mb-2">
            <h3 class="text-xs font-semibold text-muted-fg uppercase tracking-wider">{{ month_label }}</h3>
        </div>
        <div class="grid grid-cols-7 gap-0.5">
            <!-- Day headers -->
            <div class="col-span-7 grid grid-cols-7 text-center text-[10px] text-muted-fg font-medium mb-0.5">
                <span class="py-0.5">Mon</span><span class="py-0.5">Tue</span><span class="py-0.5">Wed</span><span class="py-0.5">Thu</span><span class="py-0.5">Fri</span><span class="py-0.5">Sat</span><span class="py-0.5">Sun</span>
            </div>
            {% for week in calendar_weeks %}
            <div class="col-span-7 grid grid-cols-7 gap-0.5">
                {% for day in week %}
                {% if day.is_current_month %}
                <button hx-get="/search?date={{ day.date }}"
                    hx-target="#timeline"
                    hx-swap="innerHTML"
                    class="relative flex items-center justify-center w-full aspect-square text-[11px] leading-none transition-colors rounded-lg
                        {% if day.has_memos %} bg-blue-500/70 dark:bg-blue-400/50 text-white dark:text-white font-medium hover:bg-blue-500 dark:hover:bg-blue-400/70
                        {% elif day.is_today %} bg-blue-500 dark:bg-blue-400 text-white font-semibold hover:bg-blue-600 dark:hover:bg-blue-300 shadow-sm
                        {% else %} text-muted-fg hover:bg-[#e5e5e0] dark:hover:bg-[#3e4045]{% endif %}">
                    {% if day.is_today %}
                    <span class="relative z-10">{{ day.day }}</span>
                    <span class="absolute inset-0.5 rounded-lg ring-1 ring-inset ring-white/30"></span>
                    {% else %}
                    {{ day.day }}
                    {% endif %}
                </button>
                {% else %}
                <div class="w-full aspect-square"></div>
                {% endif %}
                {% endfor %}
            </div>
            {% endfor %}
        </div>
        <button hx-get="/memos-feed" hx-target="#timeline" hx-swap="innerHTML"
            class="mt-2 w-full text-center text-xs text-blue-500 hover:text-blue-600 dark:text-blue-400 py-1 transition-colors rounded-md hover:bg-blue-50 dark:hover:bg-blue-900/20">
            Show all memos
        </button>
    </div>

    <!-- Tags -->
    <div class="flex-1 px-3 py-2 overflow-y-auto">
        <div class="border-t border-border pt-3">
            <h3 class="text-xs font-semibold text-muted-fg uppercase tracking-wider mb-2">Tags</h3>
            <div class="flex flex-wrap gap-1.5">
                {% for tag in tags %}
                <button hx-get="/search?tag={{ tag.name }}"
                    hx-target="#timeline"
                    hx-swap="innerHTML"
                    class="inline-flex items-center gap-1 px-2 py-0.5 text-xs rounded-md bg-blue-50 dark:bg-blue-900/30 text-blue-600 dark:text-blue-400 hover:bg-blue-100 dark:hover:bg-blue-900/50 transition-colors">
                    #{{ tag.name }}
                    <span class="text-[10px] text-blue-400 dark:text-blue-500">{{ tag.count }}</span>
                </button>
                {% endfor %}
                {% if not tags %}
                <p class="text-xs text-muted-fg">No tags yet. Use #tag in your memos.</p>
                {% endif %}
            </div>
            <p class="text-xs text-muted-fg text-center mt-4">MinimaMemosa · Write freely</p>
        </div>
    </div>
</div>"##;

const MEMOS_FEED_TEMPLATE: &str = r##"{% for group in memo_groups %}
<div class="mb-4">
    <div class="flex items-center gap-2 mb-3">
        <span class="text-xs font-medium text-muted-fg uppercase tracking-wider">{{ group.label }}</span>
        <span class="text-xs text-muted-fg">{{ group.date }}</span>
        <div class="flex-1 border-t border-gray-100 dark:border-gray-700"></div>
        <span class="text-xs text-muted-fg font-mono">{{ group.memos|length }}</span>
    </div>
    <div class="space-y-2">
        {% for memo in group.memos %}
        {% set id = memo.id %}
        {% set content = memo.content %}
        {% set content_html = memo.content_html %}
        {% set visibility = memo.visibility %}
        {% set created_at = memo.created_at %}
        {% set created_at_relative = memo.created_at_relative %}
        {% set resources = memo.resources %}
        {% set tags = memo.tags %}
        {% include "memo_fragment" %}
        {% endfor %}
    </div>
</div>
{% endfor %}
{% if not memo_groups %}
<div class="text-center py-16">
    <p class="text-muted-fg text-sm">No memos found</p>
</div>
{% endif %}"##;

const SHARE_NOTE_TEMPLATE: &str = r##"{% extends "base" %}
{% block content %}
<div class="flex items-center justify-center min-h-screen py-10">
    <div class="w-full max-w-2xl mx-4 bg-card rounded-xl border border-border shadow-md p-6">
        <div class="flex items-center gap-2 mb-4 border-b border-border pb-3">
            <div class="avatar-initials bg-blue-100 text-blue-800 dark:bg-blue-900/30 dark:text-blue-400">
                {{ avatar }}
            </div>
            <div>
                <div class="text-sm font-semibold text-foreground">{{ username }}</div>
                <div class="text-xs text-muted-fg">Shared Note · {{ created_at }}</div>
            </div>
            {% if visibility == 'public' %}
            <span class="ml-auto inline-flex items-center px-1.5 py-0.5 rounded text-[10px] font-medium bg-green-50 text-green-700 dark:bg-green-900/20 dark:text-green-400 border border-green-100 dark:border-green-800">Public</span>
            {% elif visibility == 'protected' %}
            <span class="ml-auto inline-flex items-center px-1.5 py-0.5 rounded text-[10px] font-medium bg-amber-50 text-amber-700 dark:bg-amber-900/20 dark:text-amber-400 border border-amber-100 dark:border-amber-800">Protected</span>
            {% endif %}
        </div>
        
        {% if title %}
        <h1 class="text-xl font-bold mb-3 text-foreground">{{ title }}</h1>
        {% endif %}
        
        <div class="memo-content text-foreground text-[15px] leading-relaxed mb-4">
            {{ content_html|safe }}
        </div>
        
        {% if resources and resources|length > 0 %}
        <div class="mt-4 border border-border rounded-xl overflow-hidden bg-muted/20">
            <div class="flex items-center gap-1.5 px-3 py-1.5 border-b border-border bg-muted/30 text-[10px] font-semibold text-muted-fg uppercase tracking-wider">
                <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15.172 7l-6.586 6.586a2 2 0 102.828 2.828l6.414-6.586a4 4 0 00-5.656-5.656l-6.415 6.585a6 6 0 108.486 8.486L20.5 13"/></svg>
                Attachments ({{ resources|length }})
            </div>
        <div class="p-3 bg-card space-y-3">
                {% for res in resources %}
                    {% if res.is_image %}
                    <div class="rounded-lg overflow-hidden border border-border bg-muted/10 flex items-center justify-center" style="max-height:calc((100vw - 18rem)/4)">
                        <img src="/resources/{{ res.id }}" class="max-w-full object-contain cursor-zoom-in" style="max-height:calc((100vw - 18rem)/4)" loading="lazy" onclick="openImageModal(this.src)">
                    </div>
                    {% else %}
                    <div class="flex items-center gap-2 p-2 rounded-lg border border-border hover:bg-muted/40 transition-colors">
                        <div class="w-8 h-8 rounded border border-border overflow-hidden shrink-0 bg-muted flex items-center justify-center">
                            <svg class="w-4 h-4 text-muted-fg" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"/></svg>
                            </div>
                        <div class="min-w-0 flex-1">
                            <p class="font-medium truncate text-foreground text-[13px]">{{ res.original_name }}</p>
                            <p class="text-[10px] text-muted-fg">{{ res.mime_type }} · {{ res.size }}</p>
                        </div>
                        <a href="/resources/{{ res.id }}" download class="p-1 rounded text-muted-fg hover:text-foreground hover:bg-muted transition-colors shrink-0" title="Download">
                            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4"/></svg>
                        </a>
                    </div>
                    {% endif %}
                {% endfor %}
            </div>
        </div>
        {% endif %}
        
        {% if tags and tags|length > 0 %}
        <div class="flex flex-wrap gap-1.5 mt-4 border-t border-border pt-3">
            {% for tag in tags %}
            <span class="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-muted text-muted-fg border border-border">#{{ tag }}</span>
            {% endfor %}
        </div>
        {% endif %}
    </div>
</div>
{% endblock %}"##;

const SHARE_PASSWORD_TEMPLATE: &str = r##"{% extends "base" %}
{% block content %}
<div class="flex items-center justify-center min-h-screen">
    <div class="w-full max-w-sm mx-4 bg-card rounded-xl border border-border shadow-md p-6">
        <div class="text-center mb-6">
            <div class="inline-flex items-center justify-center w-12 h-12 rounded-full bg-amber-50 dark:bg-amber-900/20 text-amber-600 dark:text-amber-400 mb-3 border border-amber-100 dark:border-amber-800">
                <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <rect x="3" y="11" width="18" height="11" rx="2" stroke-width="2"/>
                    <path d="M7 11V7a5 5 0 0110 0v4" stroke-width="2"/>
                </svg>
            </div>
            <h1 class="text-xl font-bold text-foreground">Protected Note</h1>
            <p class="text-xs text-muted-fg mt-1">This note is password protected. Enter the password to view it.</p>
        </div>
        <form action="/share/{{ id }}" method="post" class="space-y-4">
            <div>
                <label for="password" class="block text-sm font-medium mb-1">Password</label>
                <input type="password" name="password" id="password" required autofocus
                    class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-card focus:outline-none focus:ring-2 focus:ring-blue-500">
            </div>
            {% if error %}
            <p class="text-red-500 text-sm">{{ error }}</p>
            {% endif %}
            <button type="submit"
                class="w-full py-2 px-4 bg-blue-600 text-white rounded-lg hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500 text-sm font-medium">
                Unlock Note
            </button>
        </form>
    </div>
</div>
{% endblock %}"##;

pub struct Templates {
    env: Environment<'static>,
}

impl Templates {
    pub fn new() -> Self {
        let mut env = Environment::new();
        env.add_template("base", BASE_TEMPLATE).unwrap();
        env.add_template("login", LOGIN_TEMPLATE).unwrap();
        env.add_template("register", REGISTER_TEMPLATE).unwrap();
        env.add_template("timeline", TIMELINE_TEMPLATE).unwrap();
        env.add_template("notes_panel", NOTES_PANEL_TEMPLATE).unwrap();
        env.add_template("note_detail", NOTE_DETAIL_TEMPLATE).unwrap();
        env.add_template("memo_fragment", MEMO_FRAGMENT).unwrap();
        env.add_template("memo_edit_form", MEMO_EDIT_FORM).unwrap();
        env.add_template("resources_panel", RESOURCES_PANEL_TEMPLATE).unwrap();
        env.add_template("sidebar_timeline", SIDEBAR_TIMELINE_TEMPLATE).unwrap();
        env.add_template("memos_feed", MEMOS_FEED_TEMPLATE).unwrap();
        env.add_template("share_note", SHARE_NOTE_TEMPLATE).unwrap();
        env.add_template("share_password", SHARE_PASSWORD_TEMPLATE).unwrap();
        Templates { env }
    }

    pub fn render(&self, name: &str, ctx: &serde_json::Value) -> Html<String> {
        let tmpl = self.env.get_template(name).unwrap();
        Html(tmpl.render(ctx).unwrap())
    }
}
