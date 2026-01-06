# Contributing to Dictara

Thank you for your interest in contributing to Dictara! This document provides guidelines and instructions for contributing.

## Getting Started

### Prerequisites

- **Node.js** (LTS version 20.19+ or 22.12+)
- **Rust** (stable) - [Install via rustup](https://rustup.rs)
- **npm**

#### Installing Rust (Required)

If you don't have Rust installed, run:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

After installation, restart your terminal or run:

```bash
source "$HOME/.cargo/env"
```

Verify the installation:

```bash
cargo --version
rustc --version
```

### Stopping the Production App

If you have Dictara installed from the official release, quit it before running the development version to avoid conflicts:

1. Click the Dictara icon in your menu bar
2. Select **Quit**

Or from the terminal:

```bash
pkill -x dictara
```

### Development Setup

1. Fork and clone the repository
2. Install dependencies:
   ```bash
   npm install
   ```
3. Run the development server:
   ```bash
   npm run tauri dev
   ```
   
   > **Note:** The first build takes 5-10 minutes as it compiles ~500 Rust crates. Subsequent builds are much faster.

4. The app will open automatically. You may need to:
   - Grant **Accessibility permissions** in System Settings → Privacy & Security → Accessibility
   - Configure an **OpenAI or Azure API key** in the app's preferences

## Development Workflow

### Before Submitting Changes

Always run the verification script before submitting:

```bash
npm run verify
```

This checks for TypeScript errors, linting issues, and runs tests.

### Code Style

- **TypeScript/JavaScript**: Follow the existing code style. ESLint will catch most issues.
- **Rust**: Use `cargo fmt` for formatting and `cargo clippy` for linting.

### Commit Messages

- Use clear, descriptive commit messages
- Start with a verb (Add, Fix, Update, Remove, etc.)
- Keep the first line under 72 characters

### Pull Requests

1. Create a new branch from `main`
2. Make your changes
3. Run `npm run verify` to ensure everything passes
4. Push your branch and create a Pull Request
5. Fill out the PR template
6. Wait for CI checks to pass
7. Request a review

## Reporting Issues

### Bug Reports

When reporting bugs, please include:

- Dictara version
- Operating system and version
- Steps to reproduce
- Expected vs actual behavior
- Screenshots if applicable

### Feature Requests

We welcome feature requests! Please:

- Check existing issues first
- Clearly describe the feature
- Explain the use case

## Code of Conduct

Please read and follow our [Code of Conduct](CODE_OF_CONDUCT.md).

## Questions?

Feel free to open a [Discussion](https://github.com/vitalii-zinchenko/dictara/discussions) for questions or ideas.

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
