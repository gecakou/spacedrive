const { makeMetroConfig, resolveUniqueModule } = require('@rnx-kit/metro-config');
const MetroSymlinksResolver = require('@rnx-kit/metro-resolver-symlinks');

const [SDAssetsPath, SDAssetsPathExclude] = resolveUniqueModule('@sd/assets', '.');

// We might not need this?
const [babelRuntimePath, babelRuntimeExclude] = resolveUniqueModule('@babel/runtime');

const [reactPath, reactExclude] = resolveUniqueModule('react');
const [reactSVGPath, reactSVGExclude] = resolveUniqueModule('react-native-svg');

const { getDefaultConfig } = require('expo/metro-config');
const expoDefaultConfig = getDefaultConfig(__dirname);

const metroConfig = makeMetroConfig({
	projectRoot: __dirname,
	resolver: {
		...expoDefaultConfig.resolver,
		resolveRequest: MetroSymlinksResolver(),
		extraNodeModules: {
			'@babel/runtime': babelRuntimePath,
			'@sd/assets': SDAssetsPath,
			'react': reactPath,
			'react-native-svg': reactSVGPath
		},

		blockList: [babelRuntimeExclude, reactExclude, SDAssetsPathExclude, reactSVGExclude],
		sourceExts: [...expoDefaultConfig.resolver.sourceExts, 'svg'],
		assetExts: expoDefaultConfig.resolver.assetExts.filter((ext) => ext !== 'svg')
	},
	transformer: {
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
