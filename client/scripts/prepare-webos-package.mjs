import { copyFileSync, cpSync, existsSync, mkdirSync, rmSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = join(dirname(fileURLToPath(import.meta.url)), '..');
const dist = join(root, 'dist');
const out = join(root, 'package');

if (!existsSync(dist)) {
  console.error('Run npm run build first.');
  process.exit(1);
}

rmSync(out, { recursive: true, force: true });
mkdirSync(out, { recursive: true });

cpSync(dist, out, { recursive: true });
copyFileSync(join(root, 'appinfo.json'), join(out, 'appinfo.json'));

// Icons must sit next to appinfo.json (webOS resolves paths from app root, not public/).
for (const name of ['icon.png', 'icon-large.png']) {
  const src = join(root, name);
  if (!existsSync(src)) {
    console.error(`Missing ${name} — place it in client/ next to appinfo.json`);
    process.exit(1);
  }
  copyFileSync(src, join(out, name));
}

console.log(`webOS package root ready: ${out}`);
console.log('');
console.log('Simulator (folder — no IPK):');
console.log(`  ares-launch -s 26 ${out}`);
console.log('  npm run launch:simulator');
console.log('');
console.log('TV / emulator (IPK):');
console.log('  ares-package -n package -o build');
console.log('  ares-install --device emulator build/*.ipk');
