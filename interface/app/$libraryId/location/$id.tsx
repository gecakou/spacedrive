import { useInfiniteQuery } from '@tanstack/react-query';
import { useCallback, useEffect, useMemo } from 'react';
import { stringify } from 'uuid';
import {
	ExplorerSettings,
	useLibraryContext,
	useLibraryQuery,
	useLibrarySubscription,
	useRspcLibraryContext
} from '@sd/client';
import { LocationIdParamsSchema } from '~/app/route-schemas';
import { Folder } from '~/components';
import { useKeyDeleteFile, useZodRouteParams } from '~/hooks';
import Explorer from '../Explorer';
import { ExplorerContext } from '../Explorer/Context';
import { DefaultTopBarOptions } from '../Explorer/TopBarOptions';
import { getExplorerStore, nullValuesHandler, useExplorerStore } from '../Explorer/store';
import { useExplorer } from '../Explorer/useExplorer';
import { useExplorerOrder, useExplorerSearchParams } from '../Explorer/util';
import { TopBarPortal } from '../TopBar/Portal';
import LocationOptions from './LocationOptions';

export const Component = () => {
	const [{ path }] = useExplorerSearchParams();

	const { id: locationId } = useZodRouteParams(LocationIdParamsSchema);
	const location = useLibraryQuery(['locations.get', locationId]);
	const explorerStore = useExplorerStore();
	const locationUuid = location.data && stringify(location.data?.pub_id);
	const preferences = useLibraryQuery(['preferences.get']);

	useEffect(() => {
		if (locationUuid) {
			const explorerData = preferences.data?.location?.[locationUuid]?.explorer;
			if (!explorerData) return;
			const updatedSettings = {
				...explorerStore,
				...nullValuesHandler(explorerData as ExplorerSettings),
				orderByDirection: 'Desc' //temp till types are fixed - for testing
			};
			explorerStore.reset(updatedSettings);
		}
	}, [locationUuid]);

	useEffect(() => {
		preferences.refetch.call(undefined);
	}, [locationUuid, path, preferences.refetch]);

	useLibrarySubscription(
		[
			'locations.quickRescan',
			{
				sub_path: path ?? '',
				location_id: locationId
			}
		],
		{ onData() {} }
	);

	const { items, loadMore } = useItems({ locationId });

	const explorer = useExplorer({
		items,
		loadMore,
		parent: location.data
			? {
					type: 'Location',
					location: location.data
			  }
			: undefined
	});

	useEffect(() => {
		// Using .call to silence eslint exhaustive deps warning.
		// If clearSelectedItems referenced 'this' then this wouldn't work
		explorer.resetSelectedItems.call(undefined);
	}, [explorer.resetSelectedItems, path]);

	useKeyDeleteFile(explorer.selectedItems, location.data?.id);

	return (
		<ExplorerContext.Provider value={explorer}>
			<TopBarPortal
				left={
					<div className="group flex flex-row items-center space-x-2">
						<span className="flex flex-row items-center">
							<Folder size={22} className="ml-3 mr-2 mt-[-1px] inline-block" />
							<span className="max-w-[100px] truncate text-sm font-medium">
								{path && path?.length > 1
									? getLastSectionOfPath(path)
									: location.data?.name}
							</span>
						</span>
						{location.data && (
							<LocationOptions location={location.data} path={path || ''} />
						)}
					</div>
				}
				right={<DefaultTopBarOptions />}
			/>

			<Explorer />
		</ExplorerContext.Provider>
	);
};

const useItems = ({ locationId }: { locationId: number }) => {
	const [{ path, take }] = useExplorerSearchParams();

	const ctx = useRspcLibraryContext();
	const { library } = useLibraryContext();

	const explorerState = useExplorerStore();

	const query = useInfiniteQuery({
		queryKey: [
			'search.paths',
			{
				library_id: library.uuid,
				arg: {
					order: useExplorerOrder(),
					filter: {
						locationId,
						...(explorerState.layoutMode === 'media'
							? { object: { kind: [5, 7] } }
							: { path: path ?? '' })
					},
					take
				}
			}
		] as const,
		queryFn: ({ pageParam: cursor, queryKey }) =>
			ctx.client.query([
				'search.paths',
				{
					...queryKey[1].arg,
					cursor
				}
			]),
		getNextPageParam: (lastPage) => lastPage.cursor ?? undefined,
		keepPreviousData: true,
		onSuccess: () => getExplorerStore().resetNewThumbnails()
	});

	const items = useMemo(() => query.data?.pages.flatMap((d) => d.items) || null, [query.data]);

	const loadMore = useCallback(() => {
		if (query.hasNextPage && !query.isFetchingNextPage) {
			query.fetchNextPage.call(undefined);
		}
	}, [query.hasNextPage, query.isFetchingNextPage, query.fetchNextPage]);

	return { query, items, loadMore };
};

function getLastSectionOfPath(path: string): string | undefined {
	if (path.endsWith('/')) {
		path = path.slice(0, -1);
	}
	const sections = path.split('/');
	const lastSection = sections[sections.length - 1];
	return lastSection;
}
