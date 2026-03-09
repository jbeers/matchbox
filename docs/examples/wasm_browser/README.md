# WASM Browser — Todo App

A minimal but complete browser Todo application written entirely in BoxLang and
compiled to WebAssembly. State management and DOM manipulation are handled in
BoxLang via the `js.*` browser interop global; the HTML provides the shell and
wires up events.

## Project Layout

```
wasm_browser/
├── todo.bxs    ← BoxLang source (state + DOM rendering logic)
└── index.html  ← HTML shell (imports the compiled ES module)
```

## How It Works

`matchbox --target js` compiles `todo.bxs` to an **ES module** (`todo.js` + `todo.wasm`).

- **BoxLang owns the state** (`todos` array) and DOM rendering (`renderTodos()`).  
- **JavaScript owns the events**: the `<script type="module">` block in `index.html`
  imports the BoxLang-compiled module and wires click/keyboard events to exported
  BoxLang functions (`addTodo`, `toggleTodo`, `removeTodo`).

BoxLang functions access the browser DOM through the `js.*` global:

```boxlang
list = js.document.getElementById("todo-list")
li   = js.document.createElement("li")
li.textContent = "Buy milk"
list.appendChild(li)
```

## Build

You need the `matchbox` binary and `wasm-pack` / the WASM target available.

```bash
cd docs/examples/wasm_browser
matchbox --target js todo.bxs
```

This produces two files alongside your source:

```
todo.js    ← ES module wrapper (import this in HTML)
todo.wasm  ← WebAssembly binary (loaded automatically by todo.js)
```

## Run Locally

Browsers block WASM loading over `file://` for security reasons. Serve the
directory with any local HTTP server:

```bash
# Option A — Node.js (npx, no install required)
npx serve .

# Option B — Python
python3 -m http.server 8080

# Option C — any other static file server
```

Then open the printed URL (usually `http://localhost:3000` or
`http://localhost:8080`) in your browser.

## Features

- Add a task by typing in the input and pressing **Enter** or clicking **Add**.
- Click a task text to toggle it **done / undone**.
- Click **✕** to remove a task.

## Deploy to Production

The compiled output (`todo.js` + `todo.wasm` + `index.html`) is a fully static
bundle. Upload it to any static host:

```bash
# Vercel
vercel deploy

# Netlify drag-and-drop
# Upload the wasm_browser/ folder to app.netlify.com/drop
```

Ensure your server sets the correct MIME type for `.wasm` files:

```
Content-Type: application/wasm
```

Most modern hosts (Netlify, Vercel, Cloudflare Pages) handle this automatically.

## Key Concepts

| Concept | Where to look |
|---|---|
| `js.*` DOM API | `todo.bxs` — `renderTodos()` |
| Exported BoxLang functions | `todo.bxs` — `addTodo`, `toggleTodo`, `removeTodo` |
| ES module import | `index.html` — `import { addTodo } from './todo.js'` |
| Async function calls | `index.html` — `await addTodo(text)` |
| Event delegation | `index.html` — `list.addEventListener('click', ...)` |
