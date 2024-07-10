import { BottomSheetModalProvider } from '@gorhom/bottom-sheet';
import {
	DefaultTheme,
	NavigationContainer,
	useNavigationContainerRef
} from '@react-navigation/native';
import { QueryClient } from '@tanstack/react-query';
import dayjs from 'dayjs';
import advancedFormat from 'dayjs/plugin/advancedFormat';
import duration from 'dayjs/plugin/duration';
import relativeTime from 'dayjs/plugin/relativeTime';
import * as SplashScreen from 'expo-splash-screen';
import { StatusBar } from 'expo-status-bar';
import { checkManagePermission, requestManagePermission } from 'manage-external-storage';
import { useEffect, useRef, useState } from 'react';
import { Alert, LogBox, Permission, PermissionsAndroid, Platform } from 'react-native';
import { GestureHandlerRootView } from 'react-native-gesture-handler';
import { MenuProvider } from 'react-native-popup-menu';
import { SafeAreaProvider } from 'react-native-safe-area-context';
import { useSnapshot } from 'valtio';
import {
	ClientContextProvider,
	initPlausible,
	LibraryContextProvider,
	P2PContextProvider,
	RspcProvider,
	useBridgeQuery,
	useClientContext,
	useInvalidateQuery,
	usePlausibleEvent,
	usePlausiblePageViewMonitor,
	usePlausiblePingMonitor
} from '@sd/client';

import { GlobalModals } from './components/modal/GlobalModals';
import { Toast, toastConfig } from './components/primitive/Toast';
import { useTheme } from './hooks/useTheme';
import { changeTwTheme, tw } from './lib/tailwind';
import RootNavigator from './navigation';
import OnboardingNavigator from './navigation/OnboardingNavigator';
import { P2P } from './screens/p2p/P2P';
import { currentLibraryStore } from './utils/nav';

LogBox.ignoreLogs(['Sending `onAnimatedValueUpdate` with no listeners registered.']);

dayjs.extend(advancedFormat);
dayjs.extend(relativeTime);
dayjs.extend(duration);

// changeTwTheme(getThemeStore().theme);
// TODO: Use above when light theme is ready
changeTwTheme('dark');

function AppNavigation() {
	const { libraries, library } = useClientContext();
	const plausibleEvent = usePlausibleEvent();
	const buildInfo = useBridgeQuery(['buildInfo']);

	const navRef = useNavigationContainerRef();
	const routeNameRef = useRef<string>();

	const [currentPath, setCurrentPath] = useState<string>('/');

	useEffect(() => {
		if (buildInfo?.data) {
			initPlausible({ platformType: 'mobile', buildInfo: buildInfo.data });
		}
	}, [buildInfo]);

	usePlausiblePageViewMonitor({ currentPath });
	usePlausiblePingMonitor({ currentPath });

	useEffect(() => {
		const interval = setInterval(() => {
			plausibleEvent({ event: { type: 'ping' } });
		}, 270 * 1000);

		return () => clearInterval(interval);
	}, [plausibleEvent]);

	useEffect(() => {
		if (library === null && libraries.data) {
			currentLibraryStore.id = libraries.data[0]?.uuid ?? null;
		}
	}, [library, libraries]);

	return (
		<NavigationContainer
			ref={navRef}
			onReady={() => {
				routeNameRef.current = navRef.getCurrentRoute()?.name;
			}}
			theme={{
				...DefaultTheme,
				colors: {
					...DefaultTheme.colors,
					// Default screen background
					background: 'black'
				}
			}}
			onStateChange={async () => {
				const previousRouteName = routeNameRef.current;
				const currentRouteName = navRef.getCurrentRoute()?.name;
				if (previousRouteName !== currentRouteName) {
					// Save the current route name for later comparison
					routeNameRef.current = currentRouteName;
					// Don't track onboarding screens
					if (navRef.getRootState().routeNames.includes('GetStarted')) {
						return;
					}
					currentRouteName && setCurrentPath(currentRouteName);
				}
			}}
		>
			{!library ? (
				<OnboardingNavigator />
			) : (
				<LibraryContextProvider library={library}>
					<RootNavigator />
					<GlobalModals />
				</LibraryContextProvider>
			)}
		</NavigationContainer>
	);
}

function AppContainer() {
	useTheme();
	useInvalidateQuery();

	const { id } = useSnapshot(currentLibraryStore);

	return (
		<SafeAreaProvider style={tw`flex-1 bg-black`}>
			<GestureHandlerRootView style={tw`flex-1`}>
				<MenuProvider>
					<BottomSheetModalProvider>
						<StatusBar style="light" />
						<ClientContextProvider currentLibraryId={id}>
							<P2PContextProvider>
								<P2P />
								<AppNavigation />
								<Toast config={toastConfig} />
							</P2PContextProvider>
						</ClientContextProvider>
					</BottomSheetModalProvider>
				</MenuProvider>
			</GestureHandlerRootView>
		</SafeAreaProvider>
	);
}

const queryClient = new QueryClient();

export default function App() {
	useEffect(() => {
		SplashScreen.hideAsync();
		if (Platform.OS === 'android') requestPermissions();
	}, []);

	return (
		<RspcProvider queryClient={queryClient}>
			<AppContainer />
		</RspcProvider>
	);
}

const requestPermissions = async () => {
	try {
		const granted = await PermissionsAndroid.requestMultiple([
			PermissionsAndroid.PERMISSIONS.READ_MEDIA_AUDIO,
			PermissionsAndroid.PERMISSIONS.READ_MEDIA_IMAGES,
			PermissionsAndroid.PERMISSIONS.READ_MEDIA_VIDEO
		] as Permission[]);

		if (
			granted['android.permission.READ_MEDIA_AUDIO'] === PermissionsAndroid.RESULTS.GRANTED &&
			granted['android.permission.READ_MEDIA_IMAGES'] ===
				PermissionsAndroid.RESULTS.GRANTED &&
			granted['android.permission.READ_MEDIA_VIDEO'] === PermissionsAndroid.RESULTS.GRANTED &&
			PermissionsAndroid.RESULTS.GRANTED
		) {
			const check_MANAGE_EXTERNAL_STORAGE = await checkManagePermission();

			if (!check_MANAGE_EXTERNAL_STORAGE) {
				const request = await requestManagePermission();
				if (!request) {
					Alert.alert(
						'Permission Denied',
						'MANAGE_EXTERNAL_STORAGE permission was denied. The app may not function as expected. Please enable it in the app settings.'
					);
				}
			}
		} else {
			Alert.alert(
				'Permission Denied',
				'Some permissions were denied. The app may not function as expected. Please enable them in the app settings'
			);
		}
	} catch (err) {
		console.warn(err);
	}
};
