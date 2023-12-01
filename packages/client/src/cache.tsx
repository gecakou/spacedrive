import {
	createContext,
	PropsWithChildren,
	useContext,
	useEffect,
	useMemo,
	useRef,
	useState,
	useSyncExternalStore
} from 'react';
import { proxy, snapshot, subscribe } from 'valtio';

// TODO: Use Specta to export these
export type Reference<T> = { '__type': string; '__id': string; '#type': T };
export type CacheNode<T = any> = { '__type': string; '__id': string; '#node': T };

declare global {
	interface Window {
		__REDUX_DEVTOOLS_EXTENSION__: any;
	}
}

const defaultStore = () => ({
	nodes: {} as Record<string, Record<string, unknown>>
});

type Context = ReturnType<typeof defaultStore>;

const Context = createContext<Context>(undefined!);

export function CacheProvider({ children }: PropsWithChildren) {
	const state = useMemo(() => proxy(defaultStore()), []);

	useEffect(() => {
		const devtools = window.__REDUX_DEVTOOLS_EXTENSION__.connect({});

		const unsub = devtools.subscribe((_message: any) => {
			// console.log(message);
		});

		devtools.init();
		subscribe(state, () => devtools.send('change', snapshot(state)));

		return () => {
			unsub();
			window.__REDUX_DEVTOOLS_EXTENSION__.disconnect();
		};
		// eslint-disable-next-line react-hooks/exhaustive-deps
	}, []);

	return <Context.Provider value={state}>{children}</Context.Provider>;
}

export function useCacheContext() {
	const context = useContext(Context);
	if (!context) throw new Error('Missing `CacheContext` provider!');
	return context;
}

function restore(cache: Context, subscribed: Map<string, Set<unknown>>, item: unknown): unknown {
	if (item === undefined || item === null) {
		return item;
	} else if (Array.isArray(item)) {
		return item.map((v) => restore(cache, subscribed, v));
	} else if (typeof item === 'object') {
		if ('__type' in item && '__id' in item) {
			if (typeof item.__type !== 'string') throw new Error('Invalid `__type`');
			if (typeof item.__id !== 'string') throw new Error('Invalid `__id`');
			const result = cache.nodes?.[item.__type]?.[item.__id];
			if (!result)
				throw new Error(`Missing node for id '${item.__id}' of type '${item.__type}'`);

			const v = subscribed.get(item.__type);
			if (v) {
				v.add(item.__id);
			} else {
				subscribed.set(item.__type, new Set([item.__id]));
			}

			return result;
		}

		return Object.fromEntries(
			Object.entries(item).map(([key, value]) => [key, restore(cache, subscribed, value)])
		);
	}

	return item;
}

export function useNodes(data: CacheNode[] | undefined) {
	const cache = useCacheContext();

	// `useMemo` instead of `useEffect` here is cursed but it needs to run before the `useMemo` in the `useCache` hook.
	useMemo(() => {
		updateNodes(cache, data);
	}, [cache, data]);
}

export function useNodesCallback(): (data: CacheNode[] | undefined) => void {
	const cache = useCacheContext();

	return (data) => updateNodes(cache, data);
}

function updateNodes(cache: Context, data: CacheNode[] | undefined) {
	if (!data) return;

	for (const item of data) {
		if (!('__type' in item && '__id' in item)) throw new Error('Missing `__type` or `__id`');
		if (typeof item.__type !== 'string') throw new Error('Invalid `__type`');
		if (typeof item.__id !== 'string') throw new Error('Invalid `__id`');

		const copy = { ...item } as any;
		delete copy.__type;
		delete copy.__id;

		if (!cache.nodes[item.__type]) cache.nodes[item.__type] = {};
		cache.nodes[item.__type]![item.__id] = copy;
	}
}

type UseCacheResult<T> = T extends (infer A)[]
	? UseCacheResult<A>[]
	: T extends object
	? T extends { '__type': any; '__id': string; '#type': infer U }
		? U
		: { [K in keyof T]: UseCacheResult<T[K]> }
	: T;

export function useCache<T>(data: T | undefined) {
	const cache = useCacheContext();
	const subscribed = useRef(new Map<string, Set<unknown>>()).current;
	const [i, setI] = useState(0); // TODO: Remove this

	const state = useMemo(
		() => restore(cache, subscribed, data) as UseCacheResult<T>,
		// eslint-disable-next-line react-hooks/exhaustive-deps
		[cache, data, i]
	);

	return useSyncExternalStore(
		(onStoreChange) => {
			return subscribe(cache, (ops) => {
				for (const [_, key] of ops) {
					const key_type = key[1] as string;
					const key_id = key[2] as string;

					const v = subscribed.get(key_type);
					if (v && v.has(key_id)) {
						setI((i) => i + 1);
						onStoreChange();

						break; // We only need to trigger re-render once so we can break
					}
				}
			});
		},
		() => state
	);
}
