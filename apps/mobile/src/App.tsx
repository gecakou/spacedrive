import { BottomSheetModalProvider } from '@gorhom/bottom-sheet';
import { DefaultTheme, NavigationContainer, Theme } from '@react-navigation/native';
import { createClient } from '@rspc/client';
import type { Procedures } from '@sd/client';
import { Platform, PlatformProvider, queryClient, rspc, useInvalidateQuery } from '@sd/client';
import { StatusBar } from 'expo-status-bar';
import { Linking, Platform as RNPlatform } from 'react-native';
import { DocumentDirectoryPath } from 'react-native-fs';
import { GestureHandlerRootView } from 'react-native-gesture-handler';
import { SafeAreaProvider } from 'react-native-safe-area-context';
import { useDeviceContext } from 'twrnc';

import { GlobalModals } from './containers/modals/GlobalModals';
import useLoadApp from './hooks/useLoadApp';
import { reactNativeLink } from './lib/rspcReactNativeTransport';
import tw from './lib/tailwind';
import RootNavigator from './navigation';
import OnboardingNavigator from './navigation/OnboardingNavigator';
import { useOnboardingStore } from './stores/onboardingStore';

const NavigatorTheme: Theme = {
	...DefaultTheme,
	colors: {
		...DefaultTheme.colors,
		background: tw.color('gray-650')
	}
};

function AppContainer() {
	// Enables dark mode, and screen size breakpoints, etc. for tailwind
	useDeviceContext(tw, { withDeviceColorScheme: false });

	useInvalidateQuery();

	const { isLoadingComplete } = useLoadApp();

	const { showOnboarding } = useOnboardingStore();

	if (!isLoadingComplete) {
		return null;
	} else {
		return (
			<SafeAreaProvider style={tw`flex-1 bg-gray-650`}>
				<GestureHandlerRootView style={tw`flex-1`}>
					<BottomSheetModalProvider>
						<StatusBar style="light" />
						<NavigationContainer theme={NavigatorTheme}>
							{showOnboarding ? <OnboardingNavigator /> : <RootNavigator />}
						</NavigationContainer>
						<GlobalModals />
					</BottomSheetModalProvider>
				</GestureHandlerRootView>
			</SafeAreaProvider>
		);
	}
}

const client = createClient<Procedures>({
	links: [reactNativeLink()]
});

const platform: Platform = {
	platform: 'mobile',
	getThumbnailUrlById: (casId) =>
		`${DocumentDirectoryPath}/thumbnails/${encodeURIComponent(casId)}`,
	getOs: () => Promise.resolve(RNPlatform.OS === 'ios' ? 'ios' : 'android'),
	openLink: (url) => Linking.canOpenURL(url).then((canOpen) => canOpen && Linking.openURL(url))
};

export default function App() {
	return (
		<rspc.Provider client={client} queryClient={queryClient}>
			<>
				<PlatformProvider platform={platform}>
					<AppContainer />
				</PlatformProvider>
			</>
		</rspc.Provider>
	);
}
