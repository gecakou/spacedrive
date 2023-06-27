import { useInfiniteQuery } from '@tanstack/react-query';
import { useEffect, useMemo } from 'react';
import {
	useLibraryContext,
	useLibraryQuery,
	useLibrarySubscription,
	useRspcLibraryContext
} from '@sd/client';
import { LocationIdParamsSchema } from '~/app/route-schemas';
import { Folder } from '~/components';
import {
	getExplorerStore,
	useExplorerStore,
	useExplorerTopBarOptions,
	useZodRouteParams,
	useZodSearchParams
} from '~/hooks';
import Explorer from '../Explorer';
import { useExplorerOrder, useExplorerSearchParams } from '../Explorer/util';
import { TopBarPortal } from '../TopBar/Portal';
import TopBarOptions from '../TopBar/TopBarOptions';
import LocationOptions from './LocationOptions';

export const Component = () => {
	const [{ path }] = useExplorerSearchParams();
	const { id: locationId } = useZodRouteParams(LocationIdParamsSchema);
	const { explorerViewOptions, explorerControlOptions, explorerToolOptions } =
		useExplorerTopBarOptions();

	const { data: location } = useLibraryQuery(['locations.get', locationId]);

	useLibrarySubscription(
		[
			'locations.quickRescan',
			{
				sub_path: location?.path ?? '',
				location_id: locationId
			}
		],
		{ onData() {} }
	);

	const explorerStore = getExplorerStore();

	useEffect(() => {
		explorerStore.locationId = locationId;
	}, [explorerStore, locationId]);

	const { items, loadMore } = useItems({ locationId });

	return (
		<>
			<TopBarPortal
				left={
					<div className="group flex flex-row items-center space-x-2">
						<span className="flex flex-row items-center">
							<Folder size={22} className="ml-3 mr-2 mt-[-1px] inline-block" />
							<span className="max-w-[100px] truncate text-sm font-medium">
								{path && path?.length > 1
									? getLastSectionOfPath(path)
									: location?.name}
							</span>
						</span>
						{location && <LocationOptions location={location} path={path || ''} />}
					</div>
				}
				right={
					<TopBarOptions
						options={[explorerViewOptions, explorerToolOptions, explorerControlOptions]}
					/>
				}
			/>

			<Explorer items={items} onLoadMore={loadMore} />
		</>
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

	function loadMore() {
		if (query.hasNextPage && !query.isFetchingNextPage) {
			query.fetchNextPage();
		}
	}

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
