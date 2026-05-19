#!/usr/bin/env node
const { spawnSync } = require('child_process');
const path = require('path');
const fs = require('fs');

const platform = process.platform;
const arch = process.arch;

// Mapping of platform and architecture to package names
const PACKAGES = {
  'win32-x64': 'ghostfs-win32-x64',
  'darwin-x64': 'ghostfs-darwin-x64',
  'darwin-arm64': 'ghostfs-darwin-arm64',
  'linux-x64': 'ghostfs-linux-x64'
};

const key = `${platform}-${arch}`;
const packageName = PACKAGES[key];

if (!packageName) {
  console.error(`[ghostfs] Unsupported platform/architecture: ${key}`);
  process.exit(1);
}

// Find binary location
let binaryPath = '';

// Check if package is installed in node_modules
try {
  const packageDir = path.dirname(require.resolve(`${packageName}/package.json`));
  const binaryName = platform === 'win32' ? 'ghost.exe' : 'ghost';
  binaryPath = path.join(packageDir, binaryName);
} catch (e) {
  // If not found in node_modules (e.g. running locally for testing/development),
  // check multiple potential local target directories
  const binaryName = platform === 'win32' ? 'ghost.exe' : 'ghost';
  const possibleTargets = [
    path.join(__dirname, '..', '..', 'target', 'release', binaryName),
    path.join(__dirname, '..', '..', 'target', 'debug', binaryName),
    path.join(process.env.USERPROFILE || process.env.HOME || '', '.ghostfs-build', 'debug', binaryName),
    path.join(process.env.USERPROFILE || process.env.HOME || '', '.ghostfs-build', 'release', binaryName)
  ];
  
  const found = possibleTargets.find(t => fs.existsSync(t));
  if (found) {
    binaryPath = found;
  } else {
    console.error(`[ghostfs] Could not find the native binary. Please ensure it is installed correctly.`);
    console.error(`Tried searching in package: ${packageName}`);
    console.error(`Tried searching in local paths: \n  - ${possibleTargets.join('\n  - ')}`);
    process.exit(1);
  }
}

// Forward all arguments and execute the binary
const args = process.argv.slice(2);
const result = spawnSync(binaryPath, args, { stdio: 'inherit' });

if (result.error) {
  console.error(`[ghostfs] Failed to execute native binary:`, result.error.message);
  process.exit(1);
}

process.exit(result.status ?? 0);
