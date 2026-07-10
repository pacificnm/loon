#!/usr/bin/env node
/**
 * Fixes webOS simulator paths after moving the Loon client repo.
 *
 * 1. ~/.webos/tv/simulator-config.json — directory containing
 *    webOS_TV_<version>_Simulator*.AppImage (NOT the wrapper script).
 * 2. ~/.config/webos-simulator/webos-tv-simulator-<version>.json — appEntries
 *    points at client/package/ (built app), not the Vite source tree.
 */
import { existsSync, mkdirSync, readdirSync, readFileSync, writeFileSync } from 'node:fs';
import { homedir } from 'node:os';
import { basename, dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = join(dirname(fileURLToPath(import.meta.url)), '..');
const packageDir = join(root, 'package');
const version = process.env.WEBOS_SIM_VERSION ?? '26';
const simulatorPrefix = `webOS_TV_${version}_Simulator`;

function resolveAppImagePath() {
  const fromEnv = process.env.WEBOS_SIM_APPIMAGE?.trim();
  if (fromEnv && existsSync(fromEnv)) {
    return fromEnv;
  }

  const searchRoots = [join(homedir(), 'Downloads'), homedir(), '/opt'];

  for (const base of searchRoots) {
    if (!existsSync(base)) {
      continue;
    }
    let entries;
    try {
      entries = readdirSync(base);
    } catch {
      continue;
    }

    for (const entry of entries) {
      const candidate = join(base, entry);
      if (entry.startsWith(simulatorPrefix) && entry.toLowerCase().endsWith('.appimage')) {
        return candidate;
      }
      if (!entry.startsWith(simulatorPrefix)) {
        continue;
      }
      const nestedAppImage = join(candidate, `${entry}.AppImage`);
      if (existsSync(nestedAppImage)) {
        return nestedAppImage;
      }
      try {
        const inner = readdirSync(candidate).find(
          (name) => name.startsWith(simulatorPrefix) && name.toLowerCase().endsWith('.appimage'),
        );
        if (inner) {
          return join(candidate, inner);
        }
      } catch {
        // skip
      }
    }
  }

  return null;
}

function updateAresSimulatorConfig(simulatorDir) {
  const configDir = join(homedir(), '.webos', 'tv');
  const configPath = join(configDir, 'simulator-config.json');

  let config = {};
  try {
    config = JSON.parse(readFileSync(configPath, 'utf8'));
  } catch {
    // new file
  }

  config[version] = simulatorDir;
  mkdirSync(configDir, { recursive: true });
  writeFileSync(configPath, `${JSON.stringify(config, null, 4)}\n`, 'utf8');

  console.log(`Updated ${configPath}`);
  console.log(`  "${version}": "${simulatorDir}"`);
}

function updateSimulatorAppEntries() {
  const configPath = join(
    homedir(),
    '.config',
    'webos-simulator',
    `webos-tv-simulator-${version}.json`,
  );

  if (!existsSync(configPath)) {
    console.log(`Skip appEntries (no file): ${configPath}`);
    return;
  }

  let config;
  try {
    config = JSON.parse(readFileSync(configPath, 'utf8'));
  } catch (error) {
    console.warn(`Could not read ${configPath}: ${error}`);
    return;
  }

  config.internal ??= {};
  config.internal.appEntries = [packageDir];

  writeFileSync(configPath, `${JSON.stringify(config, null, '\t')}\n`, 'utf8');
  console.log(`Updated ${configPath}`);
  console.log(`  appEntries: ["${packageDir}"]`);
}

const appImage = resolveAppImagePath();
if (!appImage) {
  console.error(
    'Could not find webOS TV Simulator AppImage.\n' +
      'Set WEBOS_SIM_APPIMAGE to the .AppImage file, e.g.:\n' +
      '  export WEBOS_SIM_APPIMAGE=~/Downloads/webOS_TV_26_Simulator_1.5.0/webOS_TV_26_Simulator_1.5.0.AppImage',
  );
  process.exit(1);
}

const simulatorDir = dirname(appImage);
console.log(`Simulator AppImage: ${appImage}`);
console.log(`Simulator directory: ${simulatorDir}`);
console.log('');

updateAresSimulatorConfig(simulatorDir);
updateSimulatorAppEntries();

console.log('');
console.log('Launch with:');
console.log('  npm run launch:simulator');
console.log('Or: ares-launch -s 26 package');
