import { useSnapshot } from 'valtio';
import { valtioPersist } from './util';

export interface DebugState {
	enabled: boolean;
	rspcLogger: boolean;
	reactQueryDevtools: 'enabled' | 'disabled' | 'invisible';
	sendTelemetry: boolean; // used for sending telemetry even if the app is in debug mode, and ONLY if client settings also allow telemetry sharing
}

export const debugState: DebugState = valtioPersist('sd-debugState', {
	enabled: globalThis.isDev,
	rspcLogger: false,
	reactQueryDevtools: globalThis.isDev ? 'invisible' : 'enabled',
	sendTelemetry: false
});

export function useDebugState() {
	return useSnapshot(debugState);
}

export function getDebugState() {
	return debugState;
}
