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
        .memo-content p { margin-bottom: 0; line-height: 1.5; }
        .memo-content p:last-child { margin-bottom: 0; }
        .tiptap-editor p { margin-bottom: 0; line-height: 1.5; }
        .tiptap-editor p:last-child { margin-bottom: 0; }
        .tiptap-editor h1 { font-size: 1.5em; font-weight: 700; margin-top: 0.75rem; margin-bottom: 0.25rem; }
        .tiptap-editor h2 { font-size: 1.25em; font-weight: 600; margin-top: 0.5rem; margin-bottom: 0.25rem; }
        .tiptap-editor h3 { font-size: 1.1em; font-weight: 600; margin-top: 0.5rem; margin-bottom: 0.25rem; }
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
        .memo-content img { max-width: 100%; border-radius: 0.5rem; margin-top: 0.5rem; margin-bottom: 0.5rem; }
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
        .tiptap-editor .ProseMirror ul[data-type="taskList"] { list-style: none; padding-left: 0; }
        .tiptap-editor .ProseMirror ul[data-type="taskList"] li { display: flex; align-items: flex-start; gap: 0.375rem; list-style: none; }
        .tiptap-editor .ProseMirror ul[data-type="taskList"] li > label { display: flex; align-items: flex-start; gap: 0.375rem; flex: 1; cursor: pointer; }
        .tiptap-editor .ProseMirror ul[data-type="taskList"] li > label input[type="checkbox"] { margin-top: 0.3rem; cursor: pointer; }
        .tiptap-editor .ProseMirror ul[data-type="taskList"] li > label p { margin: 0; flex: 1; }
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
        .tiptap-editor .ProseMirror img { max-width: 100%; height: auto; border-radius: 0.5rem; margin-top: 0.5rem; margin-bottom: 0.5rem; cursor: pointer; }
        #image-resize-menu { background: var(--card); border: 1px solid var(--border); border-radius: 0.5rem; box-shadow: 0 4px 12px rgba(0,0,0,0.15); z-index: 9999; }
        #image-resize-menu button:hover { background: var(--muted); }
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
<div id="image-resize-menu" class="hidden fixed min-w-[130px] py-1" onclick="event.stopPropagation()">
    <button type="button" onclick="setImageWidth('25%')" class="flex items-center gap-2 w-full px-3 py-1.5 text-xs text-foreground hover:bg-muted transition-colors">Quarter Width</button>
    <button type="button" onclick="setImageWidth('50%')" class="flex items-center gap-2 w-full px-3 py-1.5 text-xs text-foreground hover:bg-muted transition-colors">Half Width</button>
    <button type="button" onclick="setImageWidth('100%')" class="flex items-center gap-2 w-full px-3 py-1.5 text-xs text-foreground hover:bg-muted transition-colors">Full Width</button>
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
<div class="flex flex-col h-screen overflow-hidden">
    <!-- Header -->
    <header class="flex items-center justify-between px-6 py-2.5 border-b border-border bg-white dark:bg-gray-900 flex-shrink-0 w-full">
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

    <div class="flex flex-1 overflow-hidden">
        <!-- Icon Bar -->
    <div class="w-14 flex-shrink-0 bg-card border-r border-border flex flex-col items-center py-3 gap-2 z-20">
        <a id="icon-timeline"
            href="/app/timeline"
            class="p-2.5 rounded-xl {% if active_panel == 'timeline' %}bg-blue-100 dark:bg-blue-900/30 text-blue-600 dark:text-blue-400{% else %}text-muted-fg hover:bg-muted hover:text-foreground{% endif %} transition-colors"
            title="Timeline">
            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <rect x="3" y="4" width="18" height="16" rx="2" stroke-width="2"/>
                <line x1="8" y1="10" x2="16" y2="10" stroke-width="2"/>
                <line x1="8" y1="14" x2="14" y2="14" stroke-width="2"/>
            </svg>
        </a>
        <a id="icon-notes"
            href="/app/notes"
            class="p-2.5 rounded-xl {% if active_panel == 'notes' %}bg-blue-100 dark:bg-blue-900/30 text-blue-600 dark:text-blue-400{% else %}text-muted-fg hover:bg-muted hover:text-foreground{% endif %} transition-colors"
            title="Notes">
            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" stroke-width="2"/>
                <polyline points="14 2 14 8 20 8" stroke-width="2"/>
                <line x1="12" y1="18" x2="12" y2="12" stroke-width="2"/>
                <line x1="9" y1="15" x2="15" y2="15" stroke-width="2"/>
            </svg>
        </a>
        <a id="icon-resources"
            href="/app/resources"
            class="p-2.5 rounded-xl {% if active_panel == 'resources' %}bg-blue-100 dark:bg-blue-900/30 text-blue-600 dark:text-blue-400{% else %}text-muted-fg hover:bg-muted hover:text-foreground{% endif %} transition-colors"
            title="Resources">
            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16l4.586-4.586a2 2 0 012.828 0L16 16m-2-2l1.586-1.586a2 2 0 012.828 0L20 14m-6-6h.01M6 20h12a2 2 0 002-2V6a2 2 0 00-2-2H6a2 2 0 00-2 2v12a2 2 0 002 2z"/>
            </svg>
        </a>
    </div>

    <!-- Sidebar Panel (timeline view - search + calendar) -->
    {% if not active_panel %}{% set active_panel = 'timeline' %}{% endif %}
    <div id="sidebar-panel"
        class="w-72 flex-shrink-0 bg-sidebar border-r border-border flex-col h-full overflow-hidden {% if active_panel != 'timeline' %}hidden{% endif %}">
        <div id="sidebar-content"
            hx-trigger="load once"
            hx-get="/sidebar-timeline"
            hx-swap="innerHTML"
            class="flex flex-col h-full">
        </div>
    </div>

    <!-- Notes Panel -->
    <div id="notes-panel"
        class="w-72 flex-shrink-0 bg-sidebar border-r border-border flex-col h-full {% if active_panel == 'notes' %}{% else %}hidden{% endif %}"
        hx-trigger="load once, memoUpdated from:body"
        hx-get="/notes-panel"
        hx-swap="innerHTML"
        hx-on::after-settle="highlightActiveNote()">
    </div>

    <!-- Resources Panel -->
    <div id="resources-panel"
        class="w-72 flex-shrink-0 bg-sidebar border-r border-border flex-col h-full {% if active_panel == 'resources' %}{% else %}hidden{% endif %}"
        hx-trigger="load once"
        hx-get="/resources-feed?offset=0"
        hx-swap="innerHTML">
    </div>

    <!-- Main Content -->
    <div id="main-content" class="flex-1 flex flex-col h-full overflow-hidden min-w-0">
        <!-- Timeline View -->
        <div id="timeline-view" class="flex-1 flex flex-col overflow-hidden">
            <div class="flex-1 overflow-y-auto px-6 py-5">
                <div class="max-w-lg mx-auto">
                     <!-- Notion-style Editor -->
                     <div class="max-w-3xl mx-auto mb-8">
                         <div class="bg-white dark:bg-gray-900 rounded-lg border border-gray-200 dark:border-gray-700 shadow-sm">
                              <form id="memo-form" hx-post="/memos"
                                    hx-swap="afterbegin"
                                    hx-target="#timeline"
                                    hx-on::after-request="if(event.detail.successful){resetEditor();htmx.trigger('body','memoUpdated')}"
                                    class="memo-editor relative"
                                    ondragover="event.preventDefault(); this.classList.add('border-blue-500')"
                                    ondragleave="event.preventDefault(); this.classList.remove('border-blue-500')"
                                    ondrop="event.preventDefault(); this.classList.remove('border-blue-500'); handleDrop(event)"
                                    onsubmit="document.getElementById('memo-editor-input').value = getTiptapMarkdown();">
                                  <div class="px-8 pt-6 pb-2 relative">
                                       <div id="memo-editor"
                                          class="w-full bg-transparent text-foreground text-base leading-snug min-h-[10rem] tiptap-editor max-w-none focus:outline-none"
                                          contenteditable="false"
                                          data-placeholder="Type '/' for commands..."
                                          oninput="onFallbackInput(this)"
                                          onkeydown="onFallbackKeydown(event, this)"></div>
                                      <!-- Attachment Previews -->
                                       <div id="attachment-preview-container" class="border border-border rounded-xl bg-card overflow-hidden hidden">
                                          <div id="attachment-preview-list" class="flex flex-col"></div>
                                      </div>
                                       <input type="hidden" name="content" id="memo-editor-input" value="">
                                       <p class="text-[10px] text-[#8e8e8a] text-center mt-2 select-none">Use <kbd class="px-1 py-0.5 bg-muted border border-border rounded text-[9px] font-mono">/</kbd> slash commands or markdown syntax to format</p>
                                   </div>
                                   <!-- Slash Commands Dropdown -->
                                   <div id="slash-menu" class="hidden bg-card border border-border rounded-lg shadow-lg py-1 min-w-[260px] z-50"></div>
                                   <input type="file" id="image-upload-input" accept="image/*" multiple class="hidden" onchange="uploadFilesForEditor(this.files);this.value=''">
                                  <input type="file" id="file-upload-input" accept="*/*" multiple class="hidden" onchange="uploadFilesForEditor(this.files);this.value=''">
                                  <div class="flex items-center justify-between px-8 py-3 border-t border-border bg-gray-50 dark:bg-gray-800/50 rounded-b-lg">
                                      <div class="flex items-center gap-1">
                                          <!-- Emoji Picker -->
                                          <div class="relative">
                                              <button type="button" onclick="toggleEmojiPicker()" class="p-1.5 rounded-md text-muted-fg hover:text-foreground hover:bg-muted transition-colors" title="Insert Emoji">
                                                  <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M14.828 14.828a4 4 0 01-5.656 0M9 10h.01M15 10h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"/></svg>
                                              </button>
                                          </div>
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
                                          </div>
                                          <div class="relative">
                                              <button type="button" onclick="toggleVisDropdown(this)" class="flex items-center gap-1 px-2 py-1 rounded-md text-muted-fg hover:text-foreground hover:bg-muted transition-colors text-xs">
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
                                          <span class="text-xs text-muted-fg">Press <kbd class="px-1.5 py-0.5 bg-muted border border-border rounded text-[10px] font-mono">/</kbd> for commands</span>
                                      </div>
                                      <button type="submit" id="save-memo-btn" disabled
                                          class="py-1.5 px-5 bg-blue-600 hover:bg-blue-700 text-white text-sm font-medium rounded-lg transition-colors focus:outline-none focus:ring-2 focus:ring-blue-500 disabled:opacity-50 disabled:cursor-not-allowed disabled:hover:bg-blue-600">
                                          Save
                                      </button>
                                  </div>
                             </form>
                         </div>
                     </div>

                    <!-- Timeline -->
                    <div id="timeline" class="space-y-1"
                        {% if selected_note %}data-active-note-id="{{ selected_note.id }}"{% endif %}>
                        {% if active_panel == 'notes' and selected_note %}
                            {% set id = selected_note.id %}
                            {% set content = selected_note.content %}
                            {% set content_html = selected_note.content_html %}
                            {% set visibility = selected_note.visibility %}
                            {% set created_at = selected_note.created_at %}
                            {% set created_at_relative = selected_note.created_at_relative %}
                            {% set tags = selected_note.tags %}
                            {% set resources = selected_note.resources %}
                            {% set username = selected_note.username %}
                            {% include "memo_fragment" %}
                        {% else %}
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
                        {% if next_offset %}
                        <div id="sentinel-memos-0" class="h-4"
                             hx-get="/memos-feed?offset={{ next_offset }}"
                             hx-trigger="revealed"
                             hx-swap="outerHTML"></div>
                        {% endif %}
                        {% if not memo_groups %}
                        <div class="text-center py-16">
                            <p class="text-muted-fg text-sm">No memos yet. Write your first memo above!</p>
                        </div>
                        {% endif %}
                        {% endif %}
                    </div>
                </div>
            </div>
        </div>

        <!-- Note Detail View (hidden by default) -->
        <div id="note-detail-view" class="flex-1 flex-col overflow-y-auto px-6 py-4 hidden">
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
            var br = document.createElement('br');
            var sel = window.getSelection();
            if (!sel.rangeCount) return;
            var range = sel.getRangeAt(0);
            range.deleteContents();
            range.insertNode(br);
            range.setStartAfter(br);
            range.collapse(true);
            sel.removeAllRanges(); sel.addRange(range);
            document.getElementById('memo-editor-input').value = getEditorText();
            updateSaveButtonState();
            return;
        }
        var _sm = document.getElementById('slash-menu');
        if (_sm && !_sm.classList.contains('hidden')) {
            if (e.key === 'ArrowDown') {
                e.preventDefault();
                if (_slashSelectedIdx < _slashFilteredCommands.length - 1) _slashSelectedIdx++;
                else _slashSelectedIdx = 0;
                _highlightSlashItem();
                return;
            }
            if (e.key === 'ArrowUp') {
                e.preventDefault();
                if (_slashSelectedIdx > 0) _slashSelectedIdx--;
                else _slashSelectedIdx = _slashFilteredCommands.length - 1;
                _highlightSlashItem();
                return;
            }
            if (e.key === 'Enter') {
                e.preventDefault();
                var cmd = _slashFilteredCommands[_slashSelectedIdx];
                if (cmd) { hideSlashMenu(); applySlashCommand(cmd); }
                return;
            }
            if (e.key === 'Escape') {
                hideSlashMenu();
                return;
            }
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
            var html = ed.getHTML();
            if (html && html !== '<p></p>') {
                if (html.indexOf('?w=') >= 0) return html;
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
        { label: 'Bold', command: 'toggleBold', icon: '<svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 4h8a4 4 0 014 4 4 4 0 01-4 4H6z"/><path d="M6 12h9a4 4 0 010 8H6z"/></svg>' },
        { label: 'Italic', command: 'toggleItalic', icon: '<svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 4h6m-2 0l-6 16"/></svg>' },
        { label: 'Bullet List', command: 'toggleBulletList', insert: '- ', icon: '<svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 6h13M8 12h13M8 18h13M3 6h.01M3 12h.01M3 18h.01"/></svg>' },
        { label: 'Numbered List', command: 'toggleOrderedList', insert: '1. ', icon: '<svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5h11M9 12h11M9 19h11M5 5v.01M5 12v.01M5 19v.01"/></svg>' },
        { label: 'Code Block', command: 'toggleCodeBlock', icon: '<svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 20l4-16m4 4l4 4-4 4M6 16l-4-4 4-4"/></svg>' },
        { label: 'Blockquote', command: 'toggleBlockquote', insert: '> ', icon: '<svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 10h16M4 14h16M4 18h16"/></svg>' },
        { label: 'Todo List', command: 'toggleTaskList', insert: '- [ ] ', icon: '<svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 11l3 3L22 4"/></svg>' },
        { label: 'Table', command: 'insertTable', params: { rows: 3, cols: 3 }, insert: '| Col1 | Col2 |\n|------|------|\n| Cell | Cell |', icon: '<svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 10h18M3 14h18M3 18h18M3 6h18"/></svg>' },
        { label: 'Code', command: 'toggleCode', icon: '<svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 20l4-16m4 4l4 4-4 4M6 16l-4-4 4-4"/></svg>' },
    ];
    function applySlashCommand(cmd) {
        var ed = window.tiptapEditor;
        if (ed) {
            if (cmd.command) {
                var chain = ed.chain().focus();
                if (typeof chain[cmd.command] === 'function') {
                    var cursorPos = ed.state.selection.$anchor.pos;
                    var textBefore = ed.state.doc.textBetween(0, cursorPos, '\n', '');
                    var match = textBefore.match(/(^|\s)\/([a-z]*)$/i);
                    if (match) {
                        var slashStart = cursorPos - (match[0].length - (match[1] ? match[1].length : 0));
                        chain = chain.deleteRange({ from: slashStart, to: cursorPos });
                    }
                    if (cmd.params) {
                        chain[cmd.command](cmd.params).run();
                    } else {
                        chain[cmd.command]().run();
                    }
                }
            } else if (cmd.insert) {
                var cursorPos = ed.state.selection.$anchor.pos;
                var textBefore = ed.state.doc.textBetween(0, cursorPos, '\n', '');
                var match = textBefore.match(/(^|\s)\/([a-z]*)$/i);
                if (match) {
                    var slashStart = cursorPos - (match[0].length - (match[1] ? match[1].length : 0));
                    ed.chain().focus().deleteRange({ from: slashStart, to: cursorPos }).insertContent(cmd.insert).run();
                }
            }
        } else {
            var el = document.getElementById('memo-editor');
            if (!el) return;
            var cursor = getTextOffset(el);
            var text = getEditorText();
            var before = text.substring(0, cursor);
            var match = before.match(/(^|\s)\/([a-z]*)$/i);
            if (match) {
                var slashStart = match.index + (match[1] ? match[1].length : 0);
                var prefix = before.substring(0, slashStart);
                var after = text.substring(cursor);
                var newText = prefix + (cmd.insert || '') + after;
                el.innerText = newText;
                var newPos = slashStart + (cmd.insert || '').length;
                restoreCursor(el, Math.min(newPos, newText.length));
            }
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
    var _slashFilteredCommands = [];
    var _slashSelectedIdx = 0;
    function _highlightSlashItem() {
        var menu = document.getElementById('slash-menu');
        if (!menu) return;
        var btns = menu.querySelectorAll('button');
        btns.forEach(function(b, i) {
            b.classList.toggle('bg-muted', i === _slashSelectedIdx);
            b.classList.toggle('text-foreground', i === _slashSelectedIdx);
            if (i === _slashSelectedIdx) b.scrollIntoView({ block: 'nearest' });
        });
    }
    function hideSlashMenu() {
        var menu = document.getElementById('slash-menu');
        if (!menu) return;
        menu.classList.add('hidden');
        if (window._slashHideTimer) {
            clearTimeout(window._slashHideTimer);
            window._slashHideTimer = null;
        }
    }
    function showSlashMenu(query, _el, _view) {
        var menu = document.getElementById('slash-menu');
        _slashFilteredCommands = query ? FALLBACK_SLASH_COMMANDS.filter(function(c) { return c.label.toLowerCase().includes(query); }) : FALLBACK_SLASH_COMMANDS;
        if (!_slashFilteredCommands.length) { hideSlashMenu(); return; }
        menu.innerHTML = '';
        _slashFilteredCommands.forEach(function(cmd, i) {
            var btn = document.createElement('button');
            btn.type = 'button';
            btn.dataset.index = i;
            btn.className = 'flex items-center gap-2 w-full px-3 py-1.5 text-xs text-foreground transition-colors';
            btn.innerHTML = cmd.icon + cmd.label;
            btn.onclick = function(e) { e.stopPropagation(); e.preventDefault(); hideSlashMenu(); applySlashCommand(cmd); };
            btn.onmouseenter = function() { _slashSelectedIdx = i; _highlightSlashItem(); };
            menu.appendChild(btn);
        });
        _slashSelectedIdx = 0;
        _highlightSlashItem();
        var rect = null;
        if (_view && typeof _view.coordsAtPos === 'function') {
            try {
                var pos = _view.state.selection.$anchor.pos;
                rect = _view.coordsAtPos(pos);
            } catch(e) {}
        }
        if (!rect || (rect.top === 0 && rect.left === 0 && rect.bottom === 0 && rect.right === 0)) {
            var sel = window.getSelection();
            if (sel.rangeCount) rect = sel.getRangeAt(0).getBoundingClientRect();
        }
        if (!rect || (rect.top === 0 && rect.left === 0 && rect.bottom === 0 && rect.right === 0)) {
            var el = document.getElementById('memo-editor') || document.getElementById('memo-edit-memo-editor-{{ id }}');
            if (el) rect = el.getBoundingClientRect();
        }
        if (!rect) return;
        menu.style.cssText = 'position: fixed; visibility: hidden; display: block;';
        menu.classList.remove('hidden');
        var menuHeight = menu.offsetHeight;
        var menuWidth = menu.offsetWidth || 260;
        if (menuHeight === 0) menuHeight = 200;
        var spaceBelow = window.innerHeight - rect.bottom;
        var spaceAbove = rect.top;
        var offset = 8;
        var top;
        if (spaceBelow >= menuHeight + offset || spaceBelow >= spaceAbove) {
            top = rect.bottom + offset;
        } else {
            top = rect.top - menuHeight - offset;
        }
        top = Math.max(8, Math.min(top, window.innerHeight - menuHeight - 8));
        var left = Math.min(Math.max(rect.left - 16, 8), window.innerWidth - menuWidth - 8);
        menu.style.top = top + 'px';
        menu.style.left = left + 'px';
        menu.style.visibility = 'visible';
        menu.style.removeProperty('display');
    }

    function showImageResizeMenu(img, x, y) {
        var menu = document.getElementById('image-resize-menu');
        if (!menu) return;
        menu.style.top = Math.max(4, Math.min(y, window.innerHeight - menu.offsetHeight - 4)) + 'px';
        menu.style.left = Math.max(4, Math.min(x, window.innerWidth - 134)) + 'px';
        menu.classList.remove('hidden');
    }
    function setImageWidth(width) {
        var menu = document.getElementById('image-resize-menu');
        if (menu) menu.classList.add('hidden');
        var ed = window.tiptapEditor;
        if (!ed) return;
        var pos = window._clickedImgPos;
        if (pos !== null && pos !== undefined) {
            var node = ed.state.doc.nodeAt(pos);
            if (node && node.type.name === 'image') {
                var src = node.attrs.src || '';
                var cleanSrc = src.replace(/[?#].*$/, '');
                var pct = parseFloat(width);
                var newSrc = width === '100%' || (pct >= 100) ? cleanSrc : cleanSrc + '?w=' + pct;
                ed.chain().focus().setNodeSelection(pos).updateAttributes('image', { src: newSrc, style: 'width: ' + width }).run();
                return;
            }
        }
        ed.chain().focus().updateAttributes('image', { style: 'width: ' + width }).run();
    }
    document.addEventListener('click', function(e) {
        var menu = document.getElementById('image-resize-menu');
        if (menu && !menu.classList.contains('hidden') && !menu.contains(e.target) && e.target.tagName !== 'IMG') {
            menu.classList.add('hidden');
        }
    });
    function closeImageModal() {
        document.getElementById('image-modal').classList.add('hidden');
    }
    function highlightActiveNote() {
        var timeline = document.getElementById('timeline');
        var activeId = timeline ? timeline.getAttribute('data-active-note-id') : null;
        if (!activeId) return;
        document.querySelectorAll('#notes-panel [data-note-id]').forEach(function(el) {
            el.classList.remove('bg-blue-50', 'dark:bg-blue-900/20');
            el.querySelector('.note-title')?.classList.remove('text-blue-600', 'dark:text-blue-400');
        });
        var selected = document.querySelector('#notes-panel [data-note-id="' + activeId + '"]');
        if (selected) {
            selected.classList.add('bg-blue-50', 'dark:bg-blue-900/20');
            selected.querySelector('.note-title')?.classList.add('text-blue-600', 'dark:text-blue-400');
        }
    }
    function openNote(id) {
        var timeline = document.getElementById('timeline');
        if (timeline) timeline.setAttribute('data-active-note-id', id);
        highlightActiveNote();
        if (timeline && document.querySelector('#notes-panel:not(.hidden)')) {
            htmx.ajax('GET', '/memos/' + id + '/fragment', { target: '#timeline', swap: 'innerHTML' });
        }
    }
    function closeNote() {
        document.getElementById('note-detail-view').classList.add('hidden');
        document.getElementById('note-detail-view').innerHTML = '';
        document.getElementById('timeline-view').classList.remove('hidden');
    }

    /* ── Memo CRUD ── */
    var editorAttachments = [];
    function renderEditorAttachments() {}
    function uploadFilesForEditor(files) {
        var input = document.getElementById('image-upload-input');
        if (!input) { input = document.getElementById('file-upload-input'); }
        if (input) {
            var dt = new DataTransfer();
            for (var i = 0; i < files.length; i++) dt.items.add(files[i]);
            input.files = dt.files;
            var form = input.closest('form');
            if (form) {
                var btn = form.querySelector('button[type="submit"]');
                if (btn) btn.click();
            }
        }
    }
    function handleDrop(event) {
        var files = [];
        if (event.dataTransfer.items) {
            for (var i = 0; i < event.dataTransfer.items.length; i++) {
                if (event.dataTransfer.items[i].kind === 'file') {
                    var f = event.dataTransfer.items[i].getAsFile();
                    if (f) files.push(f);
                }
            }
        } else {
            for (var i = 0; i < event.dataTransfer.files.length; i++) {
                files.push(event.dataTransfer.files[i]);
            }
        }
        if (files.length) uploadFilesForEditor(files);
    }
    function editMemo(id) {
        var container = document.getElementById('memo-' + id);
        if (!container) return;
        container.querySelector('.memo-display').classList.add('hidden');
        var editEl = document.getElementById('memo-edit-' + id);
        editEl.classList.remove('hidden');
        editEl.setAttribute('hx-get', '/memos/' + id + '/edit');
        editEl.setAttribute('hx-trigger', 'load');
        editEl.setAttribute('hx-swap', 'innerHTML');
        htmx.process(editEl);
        htmx.trigger(editEl, 'load');
    }
    function cancelEdit(id) {
        var container = document.getElementById('memo-' + id);
        if (!container) return;
        container.querySelector('.memo-display').classList.remove('hidden');
        var editEl = document.getElementById('memo-edit-' + id);
        editEl.classList.add('hidden');
        editEl.innerHTML = '';
    }
    function deleteMemo(id) {
        if (!confirm('Delete this memo?')) return;
        var btn = document.querySelector('#memo-' + id + ' button[onclick*="deleteMemo"]');
        if (btn) btn.disabled = true;
        htmx.ajax('DELETE', '/memos/' + id, { target: '#memo-' + id, swap: 'outerHTML' });
    }
    function toggleVisDropdown(btn) {
        var menu = btn.parentElement.querySelector('.vis-dropdown-menu');
        if (!menu) return;
        menu.classList.toggle('hidden');
    }
    function selectVis(btn) {
        var dd = btn.closest('.visibility-dropdown');
        if (!dd) return;
        dd.querySelectorAll('.vis-dropdown-menu button').forEach(function(b) { b.classList.remove('bg-muted'); });
        btn.classList.add('bg-muted');
        dd.querySelector('.vis-label').innerHTML = btn.innerHTML;
        dd.querySelector('input[type="hidden"]').value = btn.dataset.visValue;
        dd.querySelector('.vis-dropdown-menu').classList.add('hidden');
    }
    function updateVisUI(dd) {
        var val = dd.dataset.vis || 'private';
        var btn = dd.querySelector('.vis-dropdown-menu button[data-vis-value="' + val + '"]');
        if (btn) selectVis(btn);
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
            var ImageExt = window.Tiptap.Image.extend({
                addAttributes() { return { src: { default: null }, alt: { default: null }, title: { default: null }, style: { default: 'width: 100%' } } },
                parseHTML() { return [{ tag: 'img[src]', getAttrs: function(dom) { var src=dom.getAttribute('src')||''; var style=dom.getAttribute('style')||'width: 100%'; var q=src.indexOf('?'); if(q>=0){var p=src.substring(q+1).split('&');for(var i=0;i<p.length;i++){var kv=p[i].split('=');if(kv[0]==='w'){var v=parseFloat(kv[1]);if(!isNaN(v)&&v>0&&v<100){style='width: '+v+'%'}}}} return{src:src,alt:dom.getAttribute('alt')||'',title:dom.getAttribute('title')||'',style:style} } }]; },
                renderHTML({node}) { var src=node.attrs.src||''; var alt=node.attrs.alt||''; var title=node.attrs.title||''; var style=node.attrs.style||'width: 100%'; return ['img',{src:src,alt:alt,title:title,style:style}] },
            });
            var LinkExt = window.Tiptap.Link;
            var TableExt = window.Tiptap.Table;
            var TableRowExt = window.Tiptap.TableRow;
            var TableCellExt = window.Tiptap.TableCell;
            var TableHeaderExt = window.Tiptap.TableHeader;
            var TaskListExt = window.Tiptap.TaskList;
            var TaskItemExt = window.Tiptap.TaskItem;

            window._clickedImgPos = null;
            mountEl.classList.remove('animate-pulse', 'bg-muted/30', 'rounded', 'shimmer-bg');
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
                     TableExt,
                     TableRowExt,
                     TableCellExt,
                     TableHeaderExt,
                     TaskListExt,
                     TaskItemExt.configure({ nested: true }),
                 ],
                editorProps: {
                    attributes: { class: 'focus:outline-none text-base leading-snug' },
                    handleDrop: function(view, event, slice, moved) {
                        if (event.dataTransfer && event.dataTransfer.items && event.dataTransfer.items.length) {
                            var files = [];
                            for (var i = 0; i < event.dataTransfer.items.length; i++) {
                                if (event.dataTransfer.items[i].kind === 'file') {
                                    var f = event.dataTransfer.items[i].getAsFile();
                                    if (f) files.push(f);
                                }
                            }
                            if (files.length) { uploadFilesForEditor(files); event.preventDefault(); return true; }
                        }
                        return false;
                    },
                    handlePaste: function(view, event) {
                        if (event.clipboardData && event.clipboardData.files && event.clipboardData.files.length) {
                            event.preventDefault(); event.stopPropagation();
                            uploadFilesForEditor(event.clipboardData.files);
                            return true;
                        }
                        return false;
                    },
                    handleClick: function(view, pos, event) {
                        if (event.target && event.target.tagName === 'IMG') {
                            event.preventDefault();
                            var imgPos = view.posAtDOM(event.target, 0);
                            if (imgPos !== null && imgPos !== undefined) pos = imgPos;
                            _clickedImgView = view;
                            _clickedImgPos = pos;
                            showImageResizeMenu(event.target, event.clientX, event.clientY);
                            return true;
                        }
                        return false;
                    },
                    handleKeyDown: function(view, event) {
                        var _sm = document.getElementById('slash-menu');
                        if (_sm && !_sm.classList.contains('hidden')) {
                            if (event.key === 'ArrowDown') {
                                event.preventDefault();
                                if (_slashSelectedIdx < _slashFilteredCommands.length - 1) _slashSelectedIdx++;
                                else _slashSelectedIdx = 0;
                                _highlightSlashItem();
                                return true;
                            }
                            if (event.key === 'ArrowUp') {
                                event.preventDefault();
                                if (_slashSelectedIdx > 0) _slashSelectedIdx--;
                                else _slashSelectedIdx = _slashFilteredCommands.length - 1;
                                _highlightSlashItem();
                                return true;
                            }
                            if (event.key === 'Enter') {
                                event.preventDefault();
                                var cmd = _slashFilteredCommands[_slashSelectedIdx];
                                if (cmd) { hideSlashMenu(); applySlashCommand(cmd); }
                                return true;
                            }
                            if (event.key === 'Escape') {
                                hideSlashMenu();
                                return true;
                            }
                            if ((event.key.length === 1 || event.key === 'Backspace') && !event.ctrlKey && !event.metaKey && !event.altKey) {
                                setTimeout(function() {
                                    var tb = view.state.doc.textBetween(0, view.state.selection.$anchor.pos, '\n', '');
                                    var m = tb.match(/(^|\s)\/([a-z]*)$/i);
                                    if (m) { showSlashMenu(m[2] || '', mountEl, view); }
                                    else { hideSlashMenu(); }
                                }, 0);
                            }
                            return false;
                        }
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
                        if (event.key === '/') {
                            setTimeout(function() { showSlashMenu('', mountEl, view); }, 0);
                            return false;
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
                    } else {
                        var html = ed.getHTML();
                        try {
                            if (html.indexOf('?w=') >= 0) {
                                document.getElementById('memo-editor-input').value = html;
                                isEmpty = false;
                            } else {
                                var ts = new TurndownService({ headingStyle: 'atx' });
                                var md2 = ts.turndown(html);
                                document.getElementById('memo-editor-input').value = md2;
                                isEmpty = md2.trim() === '';
                            }
                        } catch(e) { isEmpty = ed.getText().trim() === ''; }
                    }
                    updateSaveButtonState();
                },
            });
        } else {
            mountEl.classList.remove('animate-pulse', 'bg-muted/30', 'rounded', 'shimmer-bg');
            mountEl.setAttribute('contenteditable', 'true');
        }
    })();
    (function() {
        var dd = document.querySelector('#memo-form .visibility-dropdown');
        if (dd) updateVisUI(dd);
    })();
    document.addEventListener('click', function() { hideSlashMenu(); });
    </script>
{% endblock %}"##;

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
{% if next_offset %}
<div id="sentinel-memos-{{ offset }}" class="h-4"
     hx-get="/memos-feed?offset={{ next_offset }}"
     hx-trigger="revealed"
     hx-swap="outerHTML"></div>
{% endif %}
{% if not memo_groups and offset == 0 %}
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

const NOTES_PANEL_TEMPLATE: &str = r#"{% if partial %}
{% for note in notes %}
<div data-note-id="{{ note.id }}" onclick="openNote({{ note.id }})"
    class="p-3 rounded-lg hover:bg-muted cursor-pointer transition-colors flex gap-3 items-start justify-between border-b border-border/30">
    <div class="flex-1 min-w-0">
        <p class="note-title text-sm font-medium text-foreground truncate flex items-center gap-1.5">
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
            <span class="inline-block px-1.5 py-0.5 text-[9px] font-medium rounded bg-blue-50 dark:bg-blue-900/20 text-blue-600 dark:text-blue-400">#{{ tag }}</span>
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
{% if next_offset %}
<div id="sentinel-notes-{{ offset }}" class="h-4"
     hx-get="/notes-panel?offset={{ next_offset }}"
     hx-trigger="revealed"
     hx-swap="outerHTML"></div>
{% endif %}
{% else %}
<div class="flex flex-col h-full">
    <div class="px-4 py-3 border-b border-border flex-shrink-0">
        <h2 class="text-xs font-semibold text-muted-fg uppercase tracking-wider">Notes</h2>
    </div>
    <div class="flex-1 overflow-y-auto p-2 space-y-1">
        {% if notes %}
            {% for note in notes %}
            <div data-note-id="{{ note.id }}" onclick="openNote({{ note.id }})"
                class="p-3 rounded-lg hover:bg-muted cursor-pointer transition-colors flex gap-3 items-start justify-between border-b border-border/30">
                <div class="flex-1 min-w-0">
                    <p class="note-title text-sm font-medium text-foreground truncate flex items-center gap-1.5">
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
                        <span class="inline-block px-1.5 py-0.5 text-[9px] font-medium rounded bg-blue-50 dark:bg-blue-900/20 text-blue-600 dark:text-blue-400">#{{ tag }}</span>
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
        {% if next_offset %}
        <div id="sentinel-notes-{{ offset }}" class="h-4"
             hx-get="/notes-panel?offset={{ next_offset }}"
             hx-trigger="revealed"
             hx-swap="outerHTML"></div>
        {% endif %}
    </div>
</div>
{% endif %}"#;

const NOTE_DETAIL_TEMPLATE: &str = r#"<div>
    <a href="/app/timeline"
        class="flex items-center gap-1.5 text-sm text-muted-fg hover:text-gray-700 dark:hover:text-gray-200 mb-4 transition-colors">
        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7"/>
        </svg>
        Back to timeline
    </a>
    <div class="memo-content">{{ content_html|safe }}</div>
    
    <p class="text-xs text-gray-400 mt-4 pt-3 border-t border-border">{{ created_at }}</p>
</div>"#;

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
                    <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"/></svg>
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
    </div>
    <div class="memo-edit hidden" id="memo-edit-{{ id }}"></div>
</div>"##;

const MEMO_EDIT_FORM: &str = r##"<form id="memo-edit-form-{{ id }}" class="memo-editor mb-0 bg-card border border-border rounded-xl shadow-sm"
      hx-put="/memos/{{ id }}"
      hx-target="#memo-{{ id }}"
      hx-swap="outerHTML"
      hx-on::after-request="if(event.detail.successful){htmx.trigger('body','memoUpdated')}"
      ondragover="event.preventDefault(); this.classList.add('border-blue-500')"
      ondragleave="event.preventDefault(); this.classList.remove('border-blue-500')"
      ondrop="event.preventDefault(); this.classList.remove('border-blue-500'); handleDrop(event)"
      onsubmit="document.getElementById('memo-edit-input-{{ id }}').value = getTiptapMarkdown();">
    <div class="px-4 pt-3 pb-1 relative">
        <div id="memo-edit-memo-editor-{{ id }}"
             class="w-full bg-transparent text-foreground text-base leading-snug min-h-[6rem] tiptap-editor max-w-none focus:outline-none"
             data-placeholder="What's on your mind..."
             data-content="{{ content|e }}"
             oninput="onFallbackInput(this)"
             onkeydown="onFallbackKeydown(event, this)"></div>
        <div id="attachment-preview-container" class="border border-border rounded-xl bg-card overflow-hidden hidden">
            <div id="attachment-preview-list" class="flex flex-col"></div>
        </div>
        <input type="hidden" name="content" id="memo-edit-input-{{ id }}" value="{{ content|e }}">
        <p class="text-[10px] text-[#8e8e8a] text-center mt-2 select-none">Use <kbd class="px-1 py-0.5 bg-muted border border-border rounded text-[9px] font-mono">/</kbd> slash commands or markdown syntax to format</p>
    </div>
    <script>
    (function() {
        var mountEl = document.getElementById('memo-edit-memo-editor-{{ id }}');
        if (!mountEl) return;
        if (window.Tiptap) {
            var Editor = window.Tiptap.Editor;
            var StarterKit = window.Tiptap.StarterKit;
            var Placeholder = window.Tiptap.Placeholder;
            var Markdown = window.Tiptap.Markdown;
            var CodeBlockLowlight = window.Tiptap.CodeBlockLowlight;
            var lowlight = window.Tiptap.lowlight;
            var ImageExt = window.Tiptap.Image.extend({
                addAttributes() { return { src: { default: null }, alt: { default: null }, title: { default: null }, style: { default: 'width: 100%' } } },
                parseHTML() { return [{ tag: 'img[src]', getAttrs: function(dom) { var src=dom.getAttribute('src')||''; var style=dom.getAttribute('style')||'width: 100%'; var q=src.indexOf('?'); if(q>=0){var p=src.substring(q+1).split('&');for(var i=0;i<p.length;i++){var kv=p[i].split('=');if(kv[0]==='w'){var v=parseFloat(kv[1]);if(!isNaN(v)&&v>0&&v<100){style='width: '+v+'%'}}}} return{src:src,alt:dom.getAttribute('alt')||'',title:dom.getAttribute('title')||'',style:style} } }]; },
                renderHTML({node}) { var src=node.attrs.src||''; var alt=node.attrs.alt||''; var title=node.attrs.title||''; var style=node.attrs.style||'width: 100%'; return ['img',{src:src,alt:alt,title:title,style:style}] },
            });
            var LinkExt = window.Tiptap.Link;
            var TableExt = window.Tiptap.Table;
            var TableRowExt = window.Tiptap.TableRow;
            var TableCellExt = window.Tiptap.TableCell;
            var TableHeaderExt = window.Tiptap.TableHeader;
            var TaskListExt = window.Tiptap.TaskList;
            var TaskItemExt = window.Tiptap.TaskItem;
            mountEl.classList.remove('animate-pulse', 'bg-muted/30', 'rounded', 'shimmer-bg');
            mountEl.removeAttribute('contenteditable');
            mountEl.removeAttribute('data-empty');
            mountEl.oninput = null;
            mountEl.onkeydown = null;
            var existingMd = mountEl.getAttribute('data-content') || '';
            mountEl.removeAttribute('data-content');
            window.tiptapEditor = new Editor({
                element: mountEl,
                extensions: [
                    StarterKit.configure({ heading: { levels: [1, 2, 3] }, codeBlock: false }),
                    Placeholder.configure({ placeholder: "What's on your mind..." }),
                    Markdown,
                    CodeBlockLowlight.configure({ lowlight: lowlight }),
                    ImageExt,
                    LinkExt.configure({ openOnClick: false }),
                    TableExt,
                    TableRowExt,
                    TableCellExt,
                    TableHeaderExt,
                    TaskListExt,
                    TaskItemExt.configure({ nested: true }),
                ],
                editorProps: {
                    attributes: { class: 'focus:outline-none text-base leading-snug' },
                    handleDrop: function(view, event, slice, moved) {
                        if (event.dataTransfer && event.dataTransfer.items && event.dataTransfer.items.length) {
                            var files = [];
                            for (var i = 0; i < event.dataTransfer.items.length; i++) {
                                if (event.dataTransfer.items[i].kind === 'file') {
                                    var f = event.dataTransfer.items[i].getAsFile();
                                    if (f) files.push(f);
                                }
                            }
                            if (files.length) { uploadFilesForEditor(files); event.preventDefault(); return true; }
                        }
                        return false;
                    },
                    handlePaste: function(view, event) {
                        if (event.clipboardData && event.clipboardData.files && event.clipboardData.files.length) {
                            event.preventDefault(); event.stopPropagation();
                            uploadFilesForEditor(event.clipboardData.files);
                            return true;
                        }
                        return false;
                    },
                    handleClick: function(view, pos, event) {
                        if (event.target && event.target.tagName === 'IMG') {
                            event.preventDefault();
                            var imgPos = view.posAtDOM(event.target, 0);
                            if (imgPos !== null && imgPos !== undefined) pos = imgPos;
                            _clickedImgView = view;
                            _clickedImgPos = pos;
                            showImageResizeMenu(event.target, event.clientX, event.clientY);
                            return true;
                        }
                        return false;
                    },
                    handleKeyDown: function(view, event) {
                        var _sm = document.getElementById('slash-menu');
                        if (_sm && !_sm.classList.contains('hidden')) {
                            if (event.key === 'ArrowDown') {
                                event.preventDefault();
                                if (_slashSelectedIdx < _slashFilteredCommands.length - 1) _slashSelectedIdx++;
                                else _slashSelectedIdx = 0;
                                _highlightSlashItem();
                                return true;
                            }
                            if (event.key === 'ArrowUp') {
                                event.preventDefault();
                                if (_slashSelectedIdx > 0) _slashSelectedIdx--;
                                else _slashSelectedIdx = _slashFilteredCommands.length - 1;
                                _highlightSlashItem();
                                return true;
                            }
                            if (event.key === 'Enter') {
                                event.preventDefault();
                                var cmd = _slashFilteredCommands[_slashSelectedIdx];
                                if (cmd) { hideSlashMenu(); applySlashCommand(cmd); }
                                return true;
                            }
                            if (event.key === 'Escape') {
                                hideSlashMenu();
                                return true;
                            }
                            if ((event.key.length === 1 || event.key === 'Backspace') && !event.ctrlKey && !event.metaKey && !event.altKey) {
                                setTimeout(function() {
                                    var tb = view.state.doc.textBetween(0, view.state.selection.$anchor.pos, '\n', '');
                                    var m = tb.match(/(^|\s)\/([a-z]*)$/i);
                                    if (m) { showSlashMenu(m[2] || '', mountEl, view); }
                                    else { hideSlashMenu(); }
                                }, 0);
                            }
                            return false;
                        }
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
                        if (event.key === '/') {
                            setTimeout(function() { showSlashMenu('', mountEl, view); }, 0);
                            return false;
                        }
                        return false;
                    }
                },
                    onUpdate: function() {
                        var ed = window.tiptapEditor;
                        if (!ed) return;
                        var isEmpty = ed.isEmpty;
                        if (isEmpty) {
                            document.getElementById('memo-edit-input-{{ id }}').value = '';
                        } else {
                            var html = ed.getHTML();
                            try {
                                if (html.indexOf('?w=') >= 0) {
                                    document.getElementById('memo-edit-input-{{ id }}').value = html;
                                    isEmpty = false;
                                } else {
                                    var ts = new TurndownService({ headingStyle: 'atx' });
                                    var md2 = ts.turndown(html);
                                    document.getElementById('memo-edit-input-{{ id }}').value = md2;
                                    isEmpty = md2.trim() === '';
                                }
                            } catch(e) { isEmpty = ed.getText().trim() === ''; }
                        }
                        var btn = document.getElementById('save-memo-edit-btn-{{ id }}');
                        if (btn) btn.disabled = isEmpty;
                    },
            });
            if (existingMd && existingMd.trim()) {
                window.tiptapEditor.commands.setContent(existingMd, true);
            }
        } else {
            mountEl.classList.remove('animate-pulse', 'bg-muted/30', 'rounded', 'shimmer-bg');
            mountEl.setAttribute('contenteditable', 'true');
        }
    })();
    (function() {
        var dd = document.querySelector('#memo-edit-form-{{ id }} .visibility-dropdown');
        if (dd) updateVisUI(dd);
    })();
</script>
    <div class="flex items-center justify-between px-4 py-2 border-t border-border">
        <div class="flex items-center gap-1">
            <div class="relative">
                <button type="button" onclick="toggleEmojiPicker()" class="p-1.5 rounded-md text-muted-fg hover:text-foreground hover:bg-muted transition-colors" title="Insert Emoji">
                    <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M14.828 14.828a4 4 0 01-5.656 0M9 10h.01M15 10h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"/></svg>
                </button>
                <div id="emoji-picker" class="hidden absolute top-full left-0 mt-1 bg-card border border-border rounded-xl shadow-xl p-2 z-50 w-[280px] max-h-[200px] overflow-y-auto">
                    <div id="emoji-grid" class="grid grid-cols-7 gap-0.5 text-lg"></div>
                </div>
            </div>
            <div class="relative">
                <button type="button" onclick="togglePlusMenu()" class="p-1.5 rounded-md text-muted-fg hover:text-foreground hover:bg-muted transition-colors" title="More">
                    <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4"/></svg>
                </button>
                <div id="plus-menu" class="hidden absolute top-full left-0 mt-1 bg-card border border-border rounded-xl shadow-xl py-1 z-50 min-w-[180px]">
                    <button type="button" onclick="uploadImage()" class="flex items-center gap-2 w-full px-3 py-1.5 text-xs text-foreground hover:bg-muted transition-colors">
                        <span>&#128247;</span> Upload Image
                    </button>
                    <button type="button" onclick="uploadFile()" class="flex items-center gap-2 w-full px-3 py-1.5 text-xs text-foreground hover:bg-muted transition-colors">
                        <span>&#128206;</span> Upload File
                    </button>
                    <button type="button" id="record-audio-btn" onclick="toggleAudioRecording()" class="flex items-center gap-2 w-full px-3 py-1.5 text-xs text-foreground hover:bg-muted transition-colors">
                        <span>&#127908;</span><span id="record-label">Record Audio</span>
                    </button>
                    <button type="button" onclick="toggleLinkMemo()" class="flex items-center gap-2 w-full px-3 py-1.5 text-xs text-foreground hover:bg-muted transition-colors">
                        <span>&#128279;</span> Link Memo
                    </button>
                </div>
                <div id="link-memo-dropdown" class="hidden absolute top-full left-0 mt-1 bg-card border border-border rounded-xl shadow-xl z-50 w-[250px]">
                    <div class="p-2"><input type="text" id="link-memo-search" placeholder="Search memos..." oninput="searchLinkMemos(this.value)" class="w-full px-2 py-1.5 text-xs bg-muted border border-border rounded-lg focus:outline-none focus:ring-1 focus:ring-blue-500"></div>
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
        <div class="flex items-center gap-2">
            <button type="button" onclick="cancelEdit({{ id }})"
                class="py-1.5 px-4 bg-transparent hover:bg-muted text-foreground text-sm font-medium rounded-lg transition-colors">
                Cancel
            </button>
            <button type="submit" id="save-memo-edit-btn-{{ id }}" disabled
                class="py-1.5 px-4 bg-blue-600 hover:bg-blue-700 text-white text-sm font-medium rounded-lg transition-colors focus:outline-none focus:ring-2 focus:ring-blue-500 disabled:opacity-50 disabled:cursor-not-allowed disabled:hover:bg-blue-600">
                Save
            </button>
        </div>
    </div>
</form>"##;

const RESOURCES_PANEL_TEMPLATE: &str = r##"{% if partial %}
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
{% if next_offset %}
<div id="sentinel-res-{{ offset }}" class="h-4"
     hx-get="/resources-feed?offset={{ next_offset }}"
     hx-trigger="revealed"
     hx-swap="outerHTML"></div>
{% endif %}
{% else %}
<div class="flex flex-col h-full">
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
        {% if next_offset %}
        <div id="sentinel-res-{{ offset }}" class="h-4"
             hx-get="/resources-feed?offset={{ next_offset }}"
             hx-trigger="revealed"
             hx-swap="outerHTML"></div>
        {% endif %}
    </div>
</div>
{% endif %}"##;

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
