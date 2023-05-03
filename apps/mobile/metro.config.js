const { makeMetroConfig, resolveUniqueModule, exclusionList } = require('@rnx-kit/metro-config');

const path = require('path');

// Needed for transforming svgs from @sd/assets
const [reactSVGPath, reactSVGExclude] = resolveUniqueModule('react-native-svg');
const [rspcClientv2Path, rspcClientv2Exclude] = resolveUniqueModule('@rspc/client');

const { getDefaultConfig } = require('expo/metro-config');
const expoDefaultConfig = getDefaultConfig(__dirname);

const projectRoot = __dirname;
const workspaceRoot = path.resolve(projectRoot, '../..');

const metroConfig = makeMetroConfig({
	...expoDefaultConfig,
	projectRoot,
	watchFolders: [workspaceRoot],
	resolver: {
		unstable_enablePackageExports: true,
		...expoDefaultConfig.resolver,
		extraNodeModules: {
			'react-native-svg': reactSVGPath,
			'@rspc/client/v2': rspcClientv2Path
		},
		blockList: exclusionList([reactSVGExclude, rspcClientv2Exclude]),
		sourceExts: [...expoDefaultConfig.resolver.sourceExts, 'svg'],
		assetExts: expoDefaultConfig.resolver.assetExts.filter((ext) => ext !== 'svg'),
		disableHierarchicalLookup: true,
		nodeModulesPaths: [
			path.resolve(projectRoot, 'node_modules'),
			path.resolve(workspaceRoot, 'node_modules')
		],
		resolveRequest: (context, moduleName, platform) => {
			if (moduleName.startsWith('@rspc/client/v2')) {
				// Logic to resolve the module name to a file path...
				// NOTE: Throw an error if there is no resolution.
				return {
					filePath: rspcClientv2Path,
					type: 'sourceFile'
				};
			}
			// Optionally, chain to the standard Metro resolver.
			return context.resolveRequest(context, moduleName, platform);
		}
	},
	transformer: {
		...expoDefaultConfig.transformer,
		// Metro default is "uglify-es" but terser should be faster and has better defaults.
		minifierPath: 'metro-minify-terser',
		minifierConfig: {
			compress: {
				drop_console: true,
				// Sometimes improves performance?
				reduce_funcs: false
			},
			format: {
				ascii_only: true,
				wrap_iife: true,
				quote_style: 3
			}
		},
		getTransformOptions: async () => ({
			transform: {
				experimentalImportSupport: false,
				inlineRequires: true
			}
		}),
		babelTransformerPath: require.resolve('react-native-svg-transformer')
	}
});

module.exports = metroConfig;
