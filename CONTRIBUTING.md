# Contributing to Component Reborn

Thank you for your interest in contributing to Component Reborn! This document provides guidelines and instructions for contributing.

## Code of Conduct

Please be respectful and constructive in all interactions. We welcome contributors of all experience levels.

## Getting Started

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (1.70 or later)
- Git

### Development Setup

1. Clone the repository:
   ```bash
   git clone https://github.com/componentjs/component
   cd component
   ```

2. Build the project:
   ```bash
   cargo build
   ```

3. Run tests:
   ```bash
   cargo test
   ```

4. Run the CLI:
   ```bash
   cargo run -- --help
   ```

## Development Workflow

### Code Style

We use rustfmt for code formatting:

```bash
cargo fmt
```

And clippy for linting:

```bash
cargo clippy
```

Please ensure your code passes both before submitting a PR.

### Testing

- Write tests for new functionality
- Ensure all existing tests pass
- Use `cargo test` to run the test suite

### Commit Messages

We follow conventional commits:

- `feat:` - New features
- `fix:` - Bug fixes
- `docs:` - Documentation changes
- `refactor:` - Code refactoring
- `test:` - Test additions or changes
- `chore:` - Build process or auxiliary tool changes

Example: `feat: add CSS module support`

## Project Structure

```
src/
├── main.rs          # CLI entry point
├── lib.rs           # Library exports
├── cli/             # CLI commands (build, dev, init)
├── bundler/         # Core bundler logic
│   ├── graph.rs     # Module dependency graph
│   └── chunk.rs     # Code splitting
├── config/          # Configuration parsing
├── resolver/        # Module resolution
├── transform/       # Code transformations (TS, JSX, CSS)
├── server/          # Development server + HMR
│   └── hmr.rs       # Hot Module Replacement
├── plugins/         # Plugin system
└── utils/           # Utility functions
```

## Areas for Contribution

### Milestone 2 - Module Graph & CSS
- Full CSS processing with lightningcss
- Multiple entrypoint support
- Asset manifest generation

### Milestone 3 - Dev Server + HMR
- Incremental rebuilds
- Hot module replacement for JavaScript
- CSS hot reloading

### Milestone 4 - Plugins & DX
- Framework plugins (React Fast Refresh, Vue HMR, Svelte)
- Better error messages and overlays
- Performance optimizations

### Other Areas
- Documentation improvements
- Test coverage
- Bug fixes
- Performance optimizations

## Submitting Changes

1. Fork the repository
2. Create a feature branch: `git checkout -b feat/my-feature`
3. Make your changes
4. Run tests and linting: `cargo test && cargo clippy`
5. Commit your changes with a descriptive message
6. Push to your fork
7. Open a Pull Request

## Questions?

Feel free to open an issue for questions, bug reports, or feature requests.

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
