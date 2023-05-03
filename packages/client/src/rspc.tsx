import { ProcedureDef, inferProcedureResult, inferSubscriptionResult } from '@rspc/client';
import { AlphaRSPCError, initRspc } from '@rspc/client/v2';
import { Context, createReactQueryHooks } from '@rspc/react/v2';
import { QueryClient } from '@tanstack/react-query';
import { PropsWithChildren, createContext, useContext } from 'react';
import { LibraryArgs, Procedures } from './core';
import { currentLibraryCache } from './hooks';

type NonLibraryProcedure<T extends keyof Procedures> =
	| Exclude<Procedures[T], { input: LibraryArgs<any> }>
	| Extract<Procedures[T], { input: never }>;

type LibraryProcedures<T extends keyof Procedures> = Exclude<
	Extract<Procedures[T], { input: LibraryArgs<any> }>,
	{ input: never }
>;

type StripLibraryArgsFromInput<T extends ProcedureDef> = T extends any
	? T['input'] extends LibraryArgs<infer E>
		? {
				key: T['key'];
				input: E;
				result: T['result'];
		  }
		: never
	: never;

type NonLibraryProceduresDef = {
	queries: NonLibraryProcedure<'queries'>;
	mutations: NonLibraryProcedure<'mutations'>;
	subscriptions: NonLibraryProcedure<'subscriptions'>;
};

type LibraryProceduresDef = {
	queries: StripLibraryArgsFromInput<LibraryProcedures<'queries'>>;
	mutations: StripLibraryArgsFromInput<LibraryProcedures<'mutations'>>;
	subscriptions: StripLibraryArgsFromInput<LibraryProcedures<'subscriptions'>>;
};

const context = createContext<Context<LibraryProceduresDef>>(undefined!);

export const useRspcLibraryContext = () => useContext(context);

export const rspc = initRspc<Procedures>({
	links: globalThis.rspcLinks
});
export const rspc2 = initRspc<Procedures>({
	links: globalThis.rspcLinks
}); // TODO: Removing this?

const nonLibraryClient = rspc.dangerouslyHookIntoInternals<NonLibraryProceduresDef>();
// @ts-expect-error // TODO: idk
const nonLibraryHooks = createReactQueryHooks(nonLibraryClient, {
	// context // TODO: Shared context
});

const libraryClient = rspc2.dangerouslyHookIntoInternals<LibraryProceduresDef>({
	mapQueryKey: (keyAndInput) => {
		const libraryId = currentLibraryCache.id;
		if (libraryId === null)
			throw new Error('Attempted to do library operation with no library set!');
		return [keyAndInput[0], { library_id: libraryId, arg: keyAndInput[1] ?? null }];
	}
});
// @ts-expect-error // TODO: idk
const libraryHooks = createReactQueryHooks(libraryClient, {
	context
});

// TODO: Allow both hooks to use a unified context -> Right now they override each others local state
export function RspcProvider({
	queryClient,
	children
}: PropsWithChildren<{ queryClient: QueryClient }>) {
	return (
		<libraryHooks.Provider client={libraryClient as any} queryClient={queryClient}>
			<nonLibraryHooks.Provider client={nonLibraryClient as any} queryClient={queryClient}>
				{children as any}
			</nonLibraryHooks.Provider>
		</libraryHooks.Provider>
	);
}

export const useBridgeQuery = nonLibraryHooks.useQuery;
export const useBridgeMutation = nonLibraryHooks.useMutation;
export const useBridgeSubscription = nonLibraryHooks.useSubscription;
export const useLibraryQuery = libraryHooks.useQuery;
export const useLibraryMutation = libraryHooks.useMutation;
export const useLibrarySubscription = libraryHooks.useSubscription;

export function useInvalidateQuery() {
	const context = nonLibraryHooks.useContext();
	useBridgeSubscription(['invalidation.listen'], {
		onData: (ops) => {
			for (const op of ops) {
				const key = [op.key];
				if (op.arg !== null) {
					key.concat(op.arg);
				}

				if (op.result !== null) {
					context.queryClient.setQueryData(key, op.result);
				} else {
					context.queryClient.invalidateQueries(key);
				}
			}
		}
	});
}

// TODO: Remove/fix this when rspc typesafe errors are working
export function extractInfoRSPCError(error: unknown) {
	if (!(error instanceof AlphaRSPCError)) return null;
	return error;
}
