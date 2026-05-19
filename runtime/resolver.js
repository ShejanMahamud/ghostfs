/// GhostFS Node.js Resolver Hook
///
/// Load with: node --require ~/.ghostfs/resolver.js app.js
/// Or set:    NODE_OPTIONS="--require ~/.ghostfs/resolver.js"
///
/// This hook intercepts `require()` calls and resolves modules
/// from the GhostFS global store instead of local node_modules.

const fs = require('fs');
const path = require('path');
const Module = require('module');

// Path to the GhostFS global store
const GHOSTFS_STORE = path.join(
  process.env.HOME || process.env.USERPROFILE,
  '.ghostfs',
  'store'
);

// Cache for resolved lockfile data
let _lockfileCache = null;
let _lockfilePath = null;

/**
 * Find and load the ghost.lock file from the current working directory
 * or any parent directory.
 */
function loadLockfile() {
  if (_lockfileCache) return _lockfileCache;

  let dir = process.cwd();
  while (true) {
    const lockPath = path.join(dir, 'ghost.lock');
    if (fs.existsSync(lockPath)) {
      try {
        const content = fs.readFileSync(lockPath, 'utf-8');
        _lockfileCache = JSON.parse(content);
        _lockfilePath = dir;
        return _lockfileCache;
      } catch (e) {
        console.error(`[ghostfs] Failed to parse ${lockPath}:`, e.message);
        return null;
      }
    }

    const parent = path.dirname(dir);
    if (parent === dir) break; // Reached root
    dir = parent;
  }

  return null;
}

/**
 * Resolve a package name to its path in the global store.
 */
function resolveFromStore(packageName) {
  const lockfile = loadLockfile();
  if (!lockfile || !lockfile.packages) return null;

  const locked = lockfile.packages[packageName];
  if (!locked) return null;

  const hash = locked.hash;
  if (!hash) return null;

  // Store path: ~/.ghostfs/store/<first2chars>/<fullhash>/
  const shard = hash.substring(0, 2);
  const storePath = path.join(GHOSTFS_STORE, shard, hash);

  if (fs.existsSync(storePath)) {
    return storePath;
  }

  return null;
}

// Save the original resolve function
const originalResolveFilename = Module._resolveFilename;

/**
 * Override Module._resolveFilename to intercept require() calls
 * and resolve from the GhostFS global store.
 */
Module._resolveFilename = function (request, parent, isMain, options) {
  // Only intercept bare specifiers (not relative or absolute paths)
  if (!request.startsWith('.') && !request.startsWith('/') && !path.isAbsolute(request)) {
    // Extract the package name (handle scoped packages)
    let packageName;
    if (request.startsWith('@')) {
      const parts = request.split('/');
      packageName = parts.slice(0, 2).join('/');
    } else {
      packageName = request.split('/')[0];
    }

    // Try to resolve from global store
    const storePath = resolveFromStore(packageName);
    if (storePath) {
      // Rewrite the request to point to the store path
      const subpath = request.substring(packageName.length);
      const resolvedRequest = subpath
        ? path.join(storePath, subpath)
        : storePath;

      try {
        return originalResolveFilename.call(this, resolvedRequest, parent, isMain, options);
      } catch (e) {
        // Fall through to default resolution
      }
    }
  }

  // Fallback to default Node.js resolution
  return originalResolveFilename.call(this, request, parent, isMain, options);
};

// Also set NODE_PATH for child processes
const lockfile = loadLockfile();
if (lockfile && lockfile.packages) {
  const storePaths = Object.values(lockfile.packages)
    .map(pkg => {
      const shard = pkg.hash.substring(0, 2);
      return path.join(GHOSTFS_STORE, shard, pkg.hash);
    })
    .filter(p => fs.existsSync(p));

  if (storePaths.length > 0) {
    const separator = process.platform === 'win32' ? ';' : ':';
    const existing = process.env.NODE_PATH || '';
    process.env.NODE_PATH = storePaths.join(separator) +
      (existing ? separator + existing : '');

    // Reinitialize module paths
    Module._initPaths();
  }
}

if (process.env.GHOSTFS_DEBUG) {
  console.error(`[ghostfs] Resolver hook loaded`);
  console.error(`[ghostfs] Store: ${GHOSTFS_STORE}`);
  console.error(`[ghostfs] Lockfile: ${_lockfilePath || 'not found'}`);
  if (lockfile) {
    console.error(`[ghostfs] Packages: ${Object.keys(lockfile.packages).length}`);
  }
}
