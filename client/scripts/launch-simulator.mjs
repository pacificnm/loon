import { spawnSync } from 'node:child_process';
import { chmodSync, existsSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = join(dirname(fileURLToPath(import.meta.url)), '..');
const appDir = join(root, 'package');
const version = process.env.WEBOS_SIM_VERSION ?? '26';
const wrapper = join(root, 'scripts', 'webos-simulator.sh');

if (!existsSync(join(appDir, 'appinfo.json'))) {
  console.error('Missing package/ — run: npm run package:webos');
  process.exit(1);
}

const args = ['-s', version];

// ares-launch cannot pass --ozone-platform=x11 / --no-sandbox; use our wrapper via -sp.
if (process.env.WEBOS_SIM_PATH) {
  args.push('-sp', process.env.WEBOS_SIM_PATH);
} else if (process.platform === 'linux' && process.env.WEBOS_SIM_NO_WRAPPER !== '1') {
  if (!existsSync(wrapper)) {
    console.error(`Missing simulator wrapper: ${wrapper}`);
    process.exit(1);
  }
  chmodSync(wrapper, 0o755);
  if (!process.env.WEBOS_SIM_APPIMAGE) {
    console.error(
      'Set WEBOS_SIM_APPIMAGE to your Simulator .AppImage, e.g.\n' +
        '  export WEBOS_SIM_APPIMAGE=~/webOS_TV_26_Simulator_1.5.0.AppImage',
    );
    process.exit(1);
  }
  args.push('-sp', wrapper);
}

args.push(appDir);

console.log(`Launching on webOS TV ${version} Simulator: ${appDir}`);
if (args.includes('-sp')) {
  const spIdx = args.indexOf('-sp');
  console.log(`Simulator launcher: ${args[spIdx + 1]}`);
}

const result = spawnSync('ares-launch', args, {
  stdio: 'inherit',
  env: process.env,
});
process.exit(result.status ?? 1);
