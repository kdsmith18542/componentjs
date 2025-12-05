  ![component reborn logo](http://i49.tinypic.com/e7nj9v.png)

# Component Reborn

> A modern, batteries-included frontend build tool, written in Rust

[![Build Status](https://github.com/componentjs/component/workflows/CI/badge.svg)](https://github.com/componentjs/component/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)

## Overview

Component Reborn is a complete rewrite of the original [componentjs](https://github.com/componentjs/component) package manager, reimagined as a modern frontend build tool. Instead of reviving the old Component ecosystem, we've taken inspiration from the original vision of a "vertically integrated frontend solution" and built something new for today's web development needs.

**One command: dev server + build + asset pipeline**

## Features

- 🚀 **Fast** - Written in Rust for maximum performance
- 📦 **ES Modules** - Native ES module support with TypeScript and JSX/TSX
- 🎨 **CSS** - CSS modules and automatic vendor prefixing
- 🔥 **HMR** - Hot Module Replacement for instant feedback
- 🛠️ **Zero Config** - Sensible defaults, fully configurable when needed
- 🔌 **Plugins** - Extensible plugin system (Vite/Rollup-style)

## Quick Start

### Installation

```bash
# Using cargo
cargo install component

# Or build from source
git clone https://github.com/componentjs/component
cd component
cargo build --release
```

### Create a New Project

```bash
# Create a new React project with TypeScript
component init --template react --typescript my-app

# Create a vanilla JavaScript project
component init my-app

# Available templates: vanilla, react, vue, svelte
```

### Development

```bash
cd my-app
component dev
```

Open http://localhost:3000 to see your app. Changes to your code will automatically trigger hot reloads.

### Production Build

```bash
component build
```

Outputs optimized bundles to the `dist/` directory.

## Configuration

Create a `component.toml` in your project root:

```toml
# Component Reborn Configuration

[project]
name = "my-app"
version = "0.1.0"

[entrypoints]
main = "src/main.tsx"
# Multiple entrypoints supported
# admin = "src/admin/main.tsx"

[output]
dir = "dist"
public_url = "/"
hash = true          # Add content hash to filenames
manifest = true      # Generate asset manifest

[features]
jsx = true
typescript = true
css_modules = true
tree_shaking = true
code_splitting = true

[dev]
port = 3000
host = "localhost"
open = false         # Auto-open browser
hmr = true           # Hot Module Replacement
```

## CLI Commands

### `component init [name]`

Initialize a new project.

```bash
component init my-app
component init --template react --typescript my-app
```

Options:
- `-t, --template <name>` - Project template (vanilla, react, vue, svelte)
- `--typescript` - Use TypeScript

### `component dev`

Start development server with HMR.

```bash
component dev
component dev --port 8080
component dev --open
```

Options:
- `-p, --port <port>` - Server port (default: 3000)
- `--host <host>` - Server host (default: localhost)
- `--open` - Open browser automatically
- `--no-hmr` - Disable hot module replacement

### `component build`

Build for production.

```bash
component build
component build --outdir dist
component build --no-minify
```

Options:
- `-o, --outdir <dir>` - Output directory
- `-m, --minify` - Enable minification (default: true)
- `--sourcemap` - Generate source maps (default: true)
- `--target <target>` - Target environment (es2020, es2021, etc.)

## Project Structure

```
my-app/
├── component.toml      # Configuration
├── package.json        # npm compatibility
├── index.html          # HTML template
├── src/
│   ├── main.tsx        # Entry point
│   ├── App.tsx         # Root component
│   └── style.css       # Global styles
└── dist/               # Build output
    ├── main.abc123.js
    ├── main.abc123.css
    └── manifest.json
```

## Roadmap

### Milestone 1 - Basic Bundler ✅
- [x] TypeScript/JavaScript bundling
- [x] Single entrypoint
- [x] Basic output bundle
- [x] Project initialization

### Milestone 2 - Module Graph & CSS
- [ ] Multiple entrypoints
- [ ] CSS handling + extraction
- [ ] Asset hashing & manifest
- [ ] Full SWC integration

### Milestone 3 - Dev Server + HMR
- [x] Local HTTP server
- [x] WebSocket HMR infrastructure
- [ ] File watching with incremental rebuilds
- [ ] Hot module replacement for JS/CSS

### Milestone 4 - Plugins & DX
- [x] Plugin API (Vite/Rollup-style)
- [ ] Framework plugins (React, Vue, Svelte)
- [ ] Error overlay
- [ ] Great error messages

## Contributing

Contributions are welcome! Please read our [Contributing Guide](CONTRIBUTING.md) for details.

```bash
# Clone the repository
git clone https://github.com/componentjs/component

# Build
cargo build

# Run tests
cargo test

# Run with example
cd examples/react-app
../../target/debug/component dev
```

## License

MIT © Component Contributors

---

## History

This project is a spiritual successor to the original [componentjs/component](https://github.com/componentjs/component), which was a pioneering frontend package manager created by TJ Holowaychuk. While the original project is archived and deprecated, Component Reborn carries forward its vision of a batteries-included frontend solution for a new generation of web development.

The original project's approach of handling "everything from package management to the build process, handling everything including HTML, JS, CSS, images, and fonts" inspired this modern rewrite. We've updated the technology stack and approach to match today's frontend development needs while maintaining the original spirit of simplicity and integration.
