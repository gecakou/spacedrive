const fs = require('node:fs');
const path = require('node:path');

const toml = require('@iarna/toml');

const { workspace, platform } = require('./const.js');

const cargoConfig = path.resolve(workspace, '.cargo/config');
const cargoConfigTempl = path.resolve(workspace, '.cargo/config.toml');

module.exports.setupFFMpegDlls = function setupDlls(FFMPEG_DIR, dev = false) {
	const ffmpegDlls = fs
		.readdirSync(path.join(env.FFMPEG_DIR, 'bin'))
		.filter((file) => file.endsWith('.dll'));

	if (dev) {
		// Ensure the target/debug directory exists
		const debugTargetDir = path.join(workspace, 'target/debug');
		fs.mkdirSync(debugTargetDir, { recursive: true });
		// Copy all DLLs from the $FFMPEG_DIR/bin to target/debug
		for (const dll of ffmpegDlls) fs.copyFileSync(dll, path.join(debugTargetDir, dll));
	}

	return ffmpegDlls;
};

module.exports.setupPlatformEnv = function setupEnv(env = {}, dev = false) {
	if (env == null || typeof env !== 'object') {
		env = {};
	}

	if (platform === 'darwin' || platform === 'win32') {
		env.PROTOC = path.join(workspace, 'target/Frameworks/bin/protoc');
		env.FFMPEG_DIR = path.join(workspace, 'target/Frameworks');

		// Check if env.PROTOC is not empty and that the value is a valid path pointing to an existing file
		if (!(env.PROTOC && fs.existsSync(env.PROTOC) && fs.statSync(env.PROTOC).isFile())) {
			console.error(`The path to protoc is invalid: ${env.PROTOC}`);
			console.error(`Did you ran the setup script: ${script}?`);
			process.exit(1);
		}

		// Check if env.FFMPEG_DIR is not empty and that the value is a valid path pointing to an existing directory
		if (
			!(
				env.FFMPEG_DIR &&
				fs.existsSync(env.FFMPEG_DIR) &&
				fs.statSync(env.FFMPEG_DIR).isDirectory()
			)
		) {
			console.error(`The path to ffmpeg is invalid: ${env.FFMPEG_DIR}`);
			console.error(`Did you ran the setup script: ${script}?`);
			process.exit(1);
		}

		// Update cargo config with the new env variables
		const cargoConf = toml.parse(fs.readFileSync(cargoConfigTempl, { encoding: 'binary' }));
		cargoConf.env = {
			...(cargoConf.env ?? {}),
			...(env ?? {}),
			PROTOC: env.PROTOC,
			FFMPEG_DIR: env.FFMPEG_DIR
		};
		fs.writeFileSync(cargoConfig, toml.stringify(cargoConf));
	}

	return env;
};
