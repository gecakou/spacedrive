import { ProcedureDef, RSPCError } from '@rspc/client';
import { internal_createReactHooksFactory } from '@rspc/react';
import { LibraryArgs, Procedures } from './core';
import { currentLibraryCache } from './hooks';
import { normiCustomHooks } from './normi';

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

let getLibraryId: () => string | null;

export const setLibraryIdGetter = (g: typeof getLibraryId) => (getLibraryId = g);

export const hooks = internal_createReactHooksFactory();

const nonLibraryHooks = hooks.createHooks<
	Procedures,
	// Normalized<NonLibraryProcedure<'queries'>>,
	// Normalized<NonLibraryProcedure<'mutations'>>
	NonLibraryProcedure<'queries'>,
	NonLibraryProcedure<'mutations'>
>({
	internal: {
		customHooks: normiCustomHooks({ contextSharing: true })
	}
});

const libraryHooks = hooks.createHooks<
	Procedures,
	// Normalized<StripLibraryArgsFromInput<LibraryProcedures<'queries'>>>,
	// Normalized<StripLibraryArgsFromInput<LibraryProcedures<'mutations'>>>,
	StripLibraryArgsFromInput<LibraryProcedures<'queries'>>,
	StripLibraryArgsFromInput<LibraryProcedures<'mutations'>>,
	StripLibraryArgsFromInput<LibraryProcedures<'subscriptions'>>
>({
	internal: {
		customHooks: normiCustomHooks({ contextSharing: true }, () => {
			return {
				mapQueryKey: (keyAndInput) => {
					const libraryId = currentLibraryCache.id;
					if (libraryId === null)
						throw new Error('Attempted to do library operation with no library set!');
					return [keyAndInput[0], { library_id: libraryId, arg: keyAndInput[1] ?? null }];
				},
				doMutation: (keyAndInput, next) => {
					const libraryId = currentLibraryCache.id;
					if (libraryId === null)
						throw new Error('Attempted to do library operation with no library set!');
					return next([
						keyAndInput[0],
						{ library_id: libraryId, arg: keyAndInput[1] ?? null }
					]);
				}
			};
		})
	}
});

export const rspc = hooks.createHooks<Procedures>();

export const useBridgeQuery = nonLibraryHooks.useQuery;
export const useBridgeMutation = nonLibraryHooks.useMutation;
export const useBridgeSubscription = nonLibraryHooks.useSubscription;
export const useLibraryQuery = libraryHooks.useQuery;
export const useLibraryMutation = libraryHooks.useMutation;

export function useInvalidateQuery() {
	const context = rspc.useContext();
	rspc.useSubscription(['invalidation.listen'], {
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

export function extractInfoRSPCError(error: unknown) {
	if (
		error == null ||
		typeof error !== 'object' ||
		!('cause' in error && error.cause instanceof RSPCError)
	)
		return null;

	// TODO: error.code property is not yet implemented in RSPCError
	// https://github.com/oscartbeaumont/rspc/blob/60a4fa93187c20bc5cb565cc6ee30b2f0903840e/packages/client/src/interop/error.ts#L59
	// So we grab it from the shape for now
	const { code } = error.cause.shape;

	return {
		code: Number.isInteger(code) ? code : 500,
		message: 'message' in error ? String(error.message) : ''
	};
}
