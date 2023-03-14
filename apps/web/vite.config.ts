import { relativeAliasResolver } from '@sd/config/vite';
import react from '@vitejs/plugin-react';
import { visualizer } from 'rollup-plugin-visualizer';
import { defineConfig } from 'vite';
import commonjs from 'vite-plugin-commonjs';
import { createHtmlPlugin } from 'vite-plugin-html';
import svg from 'vite-plugin-svgr';
import tsconfigPaths from 'vite-tsconfig-paths';
import { name, version } from './package.json';

// https://vitejs.dev/config/
export default defineConfig({
	server: {
		port: 8002
	},
	plugins: [
		tsconfigPaths(),
		react(),
		svg({ svgrOptions: { icon: true } }),
		createHtmlPlugin({
			minify: true
		}),
		visualizer({
			gzipSize: true,
			brotliSize: true
		}),
		// Used for @sd/asset `required` imports
		commonjs()
	],
	css: {
		modules: {
			localsConvention: 'camelCaseOnly'
		}
	},
	resolve: {
		alias: [relativeAliasResolver]
	},
	root: 'src',
	define: {
		pkgJson: { name, version }
	},
	build: {
		outDir: '../dist',
		assetsDir: '.'
	}
});
