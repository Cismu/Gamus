# Gamus

Gamus is a desktop application built with Tauri, Vue 3, and a modular Rust backend.

## Project Structure

The project follows a monorepo-like structure combining Rust crates and a modern frontend.

- **`src/`**: Frontend source code built with Vue 3, TypeScript, and Vite.
- **`backend/`**: The main Tauri application backend that bridges the frontend and the core logic.
- **`crates/`**: Modular Rust crates implementing the core functionality:
  - `gamus-core`: Core business logic.
  - `gamus-fs`: File system operations.
  - `gamus-metadata`: Metadata extraction and management (e.g., config, spectral analysis).
  - `gamus-scanner`: Logic for scanning media libraries.
  - `gamus-storage`: Data persistence layer.

## Getting Started

### Prerequisites

Ensure you have the following installed:

- **Rust**: [Install Rust](https://www.rust-lang.org/tools/install)
- **Node.js**: [Install Node.js](https://nodejs.org/) (Project requires Node 20+)
- **pnpm**: [Install pnpm](https://pnpm.io/installation)

### Installation

Install the frontend dependencies:

```bash
pnpm install
```

### Development

To start the development server with hot-reload:

```bash
pnpm tauri dev
# OR simply
pnpm dev
```

### Building

To build the application for production:

```bash
pnpm tauri build
```

## Recommended IDE Setup

- [VS Code](https://code.visualstudio.com/)
- [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode)
- [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)
- [Vue - Official](https://marketplace.visualstudio.com/items?itemName=Vue.volar)
