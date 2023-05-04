const { spawn } = require('./spawn.js');
const { setupFFMpegDlls, setupPlatformEnv } = require('./env.js');

setupPlatformEnv(null, true);
if (platform === 'win32') setupFFMpegDlls(env.FFMPEG_DIR, true);

let code = 0;
spawn('pnpm', ['tauri', 'dev'])
	.catch((exitCode) => {
		code = exitCode;
		console.error(`tauri dev failed with exit code ${exitCode}`);
	})
	.finally(() => {
		process.exit(code);
	});
