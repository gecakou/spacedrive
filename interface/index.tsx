import { init, Integrations } from '@sentry/browser';

import '@fontsource/inter/variable.css';

import { defaultContext } from '@tanstack/react-query';
import { ReactQueryDevtools } from '@tanstack/react-query-devtools';
import dayjs from 'dayjs';
import advancedFormat from 'dayjs/plugin/advancedFormat';
import duration from 'dayjs/plugin/duration';
import relativeTime from 'dayjs/plugin/relativeTime';
import { RouterProvider, RouterProviderProps } from 'react-router-dom';
import {
	NotificationContextProvider,
	P2PContextProvider,
	useDebugState,
	useLoadBackendFeatureFlags
} from '@sd/client';
import { TooltipProvider } from '@sd/ui';

import { P2P, useP2PErrorToast } from './app/p2p';
import { WithPrismTheme } from './components/TextViewer/prism';
import ErrorFallback, { BetterErrorBoundary } from './ErrorFallback';

export { ErrorPage } from './ErrorFallback';
export * from './app';
export * from './util/Platform';
export * from './util/keybind';
export * from './TabsContext';

dayjs.extend(advancedFormat);
dayjs.extend(relativeTime);
dayjs.extend(duration);

init({
	dsn: 'https://2fb2450aabb9401b92f379b111402dbc@o1261130.ingest.sentry.io/4504053670412288',
	environment: import.meta.env.MODE,
	defaultIntegrations: false,
	integrations: [new Integrations.HttpContext(), new Integrations.Dedupe()]
});

const Devtools = () => {
	const debugState = useDebugState();

	// The `context={defaultContext}` part is required for this to work on Windows.
	// Why, idk, don't question it
	return debugState.reactQueryDevtools !== 'disabled' ? (
		<ReactQueryDevtools
			position="bottom-right"
			context={defaultContext}
			toggleButtonProps={{
				tabIndex: -1,
				className: debugState.reactQueryDevtools === 'invisible' ? 'opacity-0' : ''
			}}
		/>
	) : null;
};

export type Router = RouterProviderProps['router'];

export const SpacedriveInterface = (props: { router: Router; routers: Router[] }) => {
	useLoadBackendFeatureFlags();
	useP2PErrorToast();

	return (
		<BetterErrorBoundary FallbackComponent={ErrorFallback}>
			<TooltipProvider>
				<P2PContextProvider>
					<NotificationContextProvider>
						<P2P />
						<Devtools />
						<WithPrismTheme />
						<RouterProvider
							key={props.routers.findIndex((r) => r === props.router)}
							router={props.router}
						/>
					</NotificationContextProvider>
				</P2PContextProvider>
			</TooltipProvider>
		</BetterErrorBoundary>
	);
};
