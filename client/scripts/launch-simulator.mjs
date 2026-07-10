import { spawnSync } from 'node:child_process';
import { chmodSync, existsSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = join(dirname(fileURLToPath(import.meta.url)), '..');
const appDir = resolve(join(root, 'package'));
const version = process.env.WEBOS_SIM_VERSION ?? '26';
const wrapper = join(root, 'scripts', 'webos-simulator.sh');

if (!existsSync(join(appDir, 'appinfo.json'))) {
  console.error('Missing package/ — run: npm run package:webos');
  process.exit(1);
}

// Linux: wrapper adds --ozone-platform=x11 --no-sandbox (ares-launch cannot).
if (process.platform === 'linux' && process.env.WEBOS_SIM_NO_WRAPPER !== '1') {
  if (!process.env.WEBOS_SIM_APPIMAGE?.trim()) {
    console.error(
      'Set WEBOS_SIM_APPIMAGE for Linux simulator launch, e.g.:\n' +
        '  export WEBOS_SIM_APPIMAGE=~/Downloads/webOS_TV_26_Simulator_1.5.0/webOS_TV_26_Simulator_1.5.0.AppImage',
    );
    process.exit(1);
  }
  if (!existsSync(wrapper)) {
    console.error(`Missing simulator wrapper: ${wrapper}`);
    process.exit(1);
  }
  chmodSync(wrapper, 0o755);
  console.log(`Launching via wrapper: ${appDir}`);
  const result = spawnSync(wrapper, [appDir], { stdio: 'inherit', env: process.env });
  process.exit(result.status ?? 1);
}

const args = ['-s', version, appDir];
console.log(`Launching on webOS TV ${version} Simulator: ${appDir}`);

const result = spawnSync('ares-launch', args, {
  stdio: 'inherit',
  env: process.env,
});
process.exit(result.status ?? 1);
