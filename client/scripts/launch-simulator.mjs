import { spawnSync } from 'node:child_process';
import { existsSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = join(dirname(fileURLToPath(import.meta.url)), '..');
const appDir = join(root, 'package');
const version = process.env.WEBOS_SIM_VERSION ?? '26';

if (!existsSync(join(appDir, 'appinfo.json'))) {
  console.error('Missing package/ — run: npm run package:webos');
  process.exit(1);
}

const args = ['-s', version];
if (process.env.WEBOS_SIM_PATH) {
  args.push('-sp', process.env.WEBOS_SIM_PATH);
}
args.push(appDir);

console.log(`Launching on webOS TV ${version} Simulator: ${appDir}`);
const result = spawnSync('ares-launch', args, { stdio: 'inherit' });
process.exit(result.status ?? 1);
