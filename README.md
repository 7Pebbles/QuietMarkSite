# Qu's Scriptless Blog

A simple, static markdown blog generator with no **JavaScript** .

## Highlights

* ✍️ Write posts in Markdown with frontmatter.
* 🌐 Generates clean, accessible HTML pages.
* 🔍 No JavaScript, no frameworks, no client-side dependencies.
* 🚫 No build tools — just one small binary that does everything.
* 📁 All output is neatly placed in a `dist/` folder, ready to deploy anywhere.

## Philosophy

This is a website, not a web app.

* Pages should load instantly.
* Content should be readable without scripts or styling.
* Everything should work offline, on slow connections, or in minimalist browsers.

## Usage

1. Write `.md` posts in the `content/` folder.
2. Run the generator:

```bash
cargo run
```

3. Serve `dist/` however you like — or run the built-in local server:

```bash
cargo run
# Visit http://localhost:3000
```

4. Clean previous builds:

```bash
cargo run -- --clean
```

## Structure

```
content/       → Your Markdown posts
template/      → Your HTML templates
dist/          → Output folder (auto-created)
```



### .gitignore

```
/dist/
/target/
*.log
*.tmp
.DS_Store
```
