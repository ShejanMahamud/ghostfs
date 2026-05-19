<p align="center">
  <h1 align="center">👻 GhostFS</h1>
  <p align="center"><strong>Dependency Virtualization Engine for JavaScript</strong></p>
  <p align="center">
    Eliminate <code>node_modules</code> forever. Packages are stored once globally and resolved at runtime.<br/>
    No install. No duplication. No waiting.
  </p>
</p>

<p align="center">
  <a href="#installation">Installation</a> •
  <a href="#quick-start">Quick Start</a> •
  <a href="#commands">Commands</a> •
  <a href="#how-it-works">How It Works</a> •
  <a href="#contributing">Contributing</a>
</p>

---

## Why GhostFS?

Every JS project creates its own `node_modules`. That means:

- 📁 **2,000–100,000 files** per project
- 💾 **GBs of duplicated packages** across projects
- 🐌 **Slow installs** every single time
- 🔗 **Broken symlinks**, antivirus indexing hell
- ⏳ **Minutes wasted** on every `git clone` + `npm install`

**GhostFS fixes all of this.** Dependencies exist globally — but not physically in your project.

```
Before (traditional):              After (GhostFS):
project/                           project/
├── src/                           ├── src/
├── package.json                   ├── ghost.json
├── node_modules/  ← 200MB+        └── ghost.lock
│   ├── react/
│   ├── next/                      ~/.ghostfs/store/  ← ONE copy, shared
│   ├── 1,847 more folders...      └── all packages by content hash
└── ...
```

---

## Installation

### Via npm (Recommended)

```bash
npm install -g ghostfs
```

### Via Shell Script (Alternative)

**Windows (PowerShell):**

```powershell
irm https://raw.githubusercontent.com/shejanmahamud/ghostfs/main/install.ps1 | iex
```

**macOS / Linux:**

```bash
curl -fsSL https://raw.githubusercontent.com/shejanmahamud/ghostfs/main/install.sh | sh
```

### From Cargo (Rust users)

```bash
cargo install ghostfs-cli
```

### From GitHub Releases

Download the latest binary for your platform from [Releases](https://github.com/shejanmahamud/ghostfs/releases).

| Platform | Binary |
|---|---|
| Windows x64 | `ghost-x86_64-pc-windows-msvc.zip` |
| macOS x64 | `ghost-x86_64-apple-darwin.tar.gz` |
| macOS ARM | `ghost-aarch64-apple-darwin.tar.gz` |
| Linux x64 | `ghost-x86_64-unknown-linux-gnu.tar.gz` |

### Build from Source

```bash
git clone https://github.com/shejanmahamud/ghostfs.git
cd ghostfs
cargo build --release
# Binary at target/release/ghost (or ghost.exe on Windows)
```

---

## Quick Start

```bash
# 1. Initialize a project
ghost init

# 2. Add dependencies
ghost add react
ghost add next
ghost add typescript -D

# 3. Install everything to the global store
ghost install

# 4. Run your app
ghost dev
```

That's it. **No `node_modules` created.** Dependencies are resolved from `~/.ghostfs/store/`.

---

## Commands

| Command | Description |
|---|---|
| `ghost init` | Create a new `ghost.json` project manifest |
| `ghost create <temp> <name>` | Scaffold a new project from a template (react, next, vite, node) |
| `ghost templates` | List available project scaffolding templates |
| `ghost add <pkg>` | Add a dependency (resolves latest from npm) |
| `ghost remove <pkg>` | Remove a dependency from `ghost.json` |
| `ghost install` | Resolve + download all deps to the global store |
| `ghost install --link` | Install and also link packages into `node_modules` |
| `ghost link` | Link global store packages into project `node_modules/` (symlinks/junctions) |
| `ghost unlink` | Remove the managed `node_modules/` directory |
| `ghost list` | Show all packages cached in the global store |
| `ghost run <script>` | Run a project script with virtual resolution |
| `ghost dev` | Shortcut for `ghost run dev` |
| `ghost status` | Show store statistics and hook installation status |
| `ghost install-hooks` | Install Node.js resolver hooks to bypass `node_modules` |
| `ghost clean` | Prune the global package store (freed space) |

---

## Zero node_modules Hook

You can run ANY Node.js project or tool without `node_modules` by installing and loading the GhostFS runtime hooks:

```bash
# 1. Install the hooks
ghost install-hooks

# 2. Run your app (CommonJS)
node --require ~/.ghostfs/runtime/resolver.js app.js

# Or ESM:
node --loader ~/.ghostfs/runtime/loader.mjs app.mjs
```

Or make it transparent for all node execution in your shell:
```bash
# Windows (PowerShell)
$env:NODE_OPTIONS="--require C:\Users\Name\.ghostfs\runtime\resolver.js"

# macOS/Linux
export NODE_OPTIONS="--require ~/.ghostfs/runtime/resolver.js"
```

## How It Works

### 1. Global Content-Addressed Store

All packages live in `~/.ghostfs/store/`, indexed by SHA-256 hash. If 100 projects use React, only **one copy** exists on disk.

### 2. Virtual Resolution

When you run `ghost install`, GhostFS:
1. Reads your `ghost.json` (or `package.json`)
2. Resolves the full dependency tree from the npm registry
3. Downloads only **new** packages (skips cached ones)
4. Stores them in the global store by content hash
5. Writes a `ghost.lock` for reproducible builds

### 3. Runtime Integration

`ghost run` and `ghost dev` set `NODE_PATH` to point at the global store, so Node.js resolves modules directly — no `node_modules` needed.

### 4. Zero-Install Projects

Clone any GhostFS project and run immediately:

```bash
git clone your-repo
cd your-repo
ghost dev   # Dependencies already in global store!
```

---

## Project Manifest (`ghost.json`)

```json
{
  "name": "my-app",
  "version": "1.0.0",
  "description": "My awesome app",
  "dependencies": {
    "react": "^19.0.0",
    "next": "^15.0.0"
  },
  "devDependencies": {
    "typescript": "^5.0.0"
  },
  "scripts": {
    "dev": "next dev",
    "build": "next build"
  }
}
```

GhostFS also reads `package.json` for compatibility.

---

## Architecture

```
┌──────────────────────────────────────────────────┐
│                  ghost CLI                        │
│  init │ add │ install │ list │ run │ dev │ status │
└──────────────┬───────────────────────────────────┘
               │
┌──────────────▼───────────────────────────────────┐
│              ghostfs-core                         │
│  Manifest │ Lockfile │ Resolver │ Installer       │
└──────┬───────────────────────────────┬───────────┘
       │                               │
┌──────▼──────────┐           ┌───────▼────────────┐
│  ghostfs-store   │           │  ghostfs-registry  │
│  Content Store   │           │  npm Registry API  │
│  SQLite Metadata │           │  Tarball Downloads  │
│  SHA-256 Hashing │           │  Async HTTP Client  │
└─────────────────┘           └────────────────────┘
```

Built with **Rust** for maximum performance. Uses **Tokio** for async I/O, **SQLite** for metadata, and **semver** for version resolution.

---

## Contributing

```bash
git clone https://github.com/shejanmahamud/ghostfs.git
cd ghostfs
cargo build
cargo test
```

PRs welcome! See [issues](https://github.com/shejanmahamud/ghostfs/issues) for good first contributions.

---

## License

MIT © [Shejan Mahamud](https://github.com/shejanmahamud)
