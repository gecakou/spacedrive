const { makeMetroConfig, resolveUniqueModule } = require('@rnx-kit/metro-config');
const MetroSymlinksResolver = require('@rnx-kit/metro-resolver-symlinks');

const [SDInterfacePath, SDInterfacePathExclude] = resolveUniqueModule('@sd/interface', '.');

const [babelRuntimePath, babelRuntimeExclude] = resolveUniqueModule('@babel/runtime');
const [reactPath, reactExclude] = resolveUniqueModule('react');

const { getDefaultConfig } = require('expo/metro-config');
const expoDefaultConfig = getDefaultConfig(__dirname);

const metroConfig = makeMetroConfig({
	projectRoot: __dirname,
	resolver: {
		resolveRequest: MetroSymlinksResolver(),
		extraNodeModules: {
			'@babel/runtime': babelRuntimePath,
			'@sd/interface': SDInterfacePath,
			'react': reactPath
		},
		blockList: [babelRuntimeExclude, reactExclude, SDInterfacePathExclude],
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
