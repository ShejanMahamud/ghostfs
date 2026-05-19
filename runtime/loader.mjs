/// GhostFS ESM Loader Hook
///
/// Load with: node --loader ~/.ghostfs/loader.mjs app.mjs
///
/// This loader intercepts ESM import() calls and resolves modules
/// from the GhostFS global store.

import { readFileSync, existsSync } from 'fs';
import { join, dirname } from 'path';
import { fileURLToPath, pathToFileURL } from 'url';
import { homedir } from 'os';

const GHOSTFS_STORE = join(homedir(), '.ghostfs', 'store');

let _lockfileCache = null;

function loadLockfile() {
  if (_lockfileCache) return _lockfileCache;

  let dir = process.cwd();
  while (true) {
    const lockPath = join(dir, 'ghost.lock');
    if (existsSync(lockPath)) {
      try {
        const content = readFileSync(lockPath, 'utf-8');
        _lockfileCache = JSON.parse(content);
        return _lockfileCache;
      } catch {
        return null;
      }
    }
    const parent = dirname(dir);
    if (parent === dir) break;
    dir = parent;
  }
  return null;
}

function resolveFromStore(packageName) {
  const lockfile = loadLockfile();
  if (!lockfile?.packages) return null;

  const locked = lockfile.packages[packageName];
  if (!locked?.hash) return null;

  const shard = locked.hash.substring(0, 2);
  const storePath = join(GHOSTFS_STORE, shard, locked.hash);

  return existsSync(storePath) ? storePath : null;
}

export async function resolve(specifier, context, nextResolve) {
  // Only handle bare specifiers
  if (
    !specifier.startsWith('.') &&
    !specifier.startsWith('/') &&
    !specifier.startsWith('file:') &&
    !specifier.startsWith('node:') &&
    !specifier.startsWith('data:')
  ) {
    let packageName;
    if (specifier.startsWith('@')) {
      const parts = specifier.split('/');
      packageName = parts.slice(0, 2).join('/');
    } else {
      packageName = specifier.split('/')[0];
    }

    const storePath = resolveFromStore(packageName);
    if (storePath) {
      const subpath = specifier.substring(packageName.length);
      const fullPath = subpath ? join(storePath, subpath) : storePath;

      // Try resolving with the rewritten path
      try {
        return await nextResolve(
          pathToFileURL(fullPath).href,
          context
        );
      } catch {
        // Try with package.json main field
        try {
          const pkgJson = join(storePath, 'package.json');
          if (existsSync(pkgJson)) {
            const pkg = JSON.parse(readFileSync(pkgJson, 'utf-8'));
            const main = pkg.module || pkg.main || 'index.js';
            const mainPath = join(storePath, main);
            return await nextResolve(
              pathToFileURL(mainPath).href,
              context
            );
          }
        } catch {
          // Fall through
        }
      }
    }
  }

  return nextResolve(specifier, context);
}
