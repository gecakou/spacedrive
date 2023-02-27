import { Integrations, init } from '@sentry/browser';
import '@fontsource/inter/variable.css';
import { defaultContext } from '@tanstack/react-query';
import { ReactQueryDevtools } from '@tanstack/react-query-devtools';
import dayjs from 'dayjs';
import advancedFormat from 'dayjs/plugin/advancedFormat';
import duration from 'dayjs/plugin/duration';
import relativeTime from 'dayjs/plugin/relativeTime';
import { ErrorBoundary } from 'react-error-boundary';
import { BrowserRouter, MemoryRouter } from 'react-router-dom';
import { useDebugState } from '@sd/client';
import { Dialogs } from '@sd/ui';
import ErrorFallback from './ErrorFallback';
import App from './app';

export * from './util/keybind';
export * from './util/Platform';
export { ErrorPage } from './ErrorFallback';

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
				className: debugState.reactQueryDevtools === 'invisible' ? 'opacity-0' : ''
			}}
		/>
	) : null;
};

export const SpacedriveInterface = ({ router }: { router: 'memory' | 'browser' }) => {
	const Router = router === 'memory' ? MemoryRouter : BrowserRouter;

	return (
		<ErrorBoundary FallbackComponent={ErrorFallback}>
			<Devtools />
			<Router>
				<App />
			</Router>
			<Dialogs />
		</ErrorBoundary>
	);
};
