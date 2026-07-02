import { spawn } from 'node:child_process';
import { chmodSync, existsSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = join(dirname(fileURLToPath(import.meta.url)), '..');
const wrapper = join(root, 'scripts', 'webos-simulator.sh');

if (!existsSync(wrapper)) {
  console.error(`Missing simulator wrapper: ${wrapper}`);
  process.exit(1);
}

if (!process.env.WEBOS_SIM_APPIMAGE) {
  console.error(
    'Set WEBOS_SIM_APPIMAGE to your Simulator .AppImage, e.g.\n' +
      '  export WEBOS_SIM_APPIMAGE=~/webOS_TV_26_Simulator_1.5.0.AppImage\n' +
      'Then launch your app from Simulator: File → Launch App → package/',
  );
  process.exit(1);
}

chmodSync(wrapper, 0o755);

console.log('Starting webOS TV Simulator with --ozone-platform=x11 --no-sandbox');
const child = spawn(wrapper, [], {
  stdio: 'inherit',
  env: process.env,
  detached: false,
});

child.on('exit', (code) => process.exit(code ?? 1));
