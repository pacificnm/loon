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
copyFileSync(join(root, 'public', 'icon.png'), join(out, 'icon.png'));
copyFileSync(join(root, 'public', 'icon-large.png'), join(out, 'icon-large.png'));

console.log(`webOS package root ready: ${out}`);
console.log('Run: ares-package package -o build');
