import { useCallback, useMemo } from 'react';
import { ObjectKindEnum, ObjectOrder, Tag, useLibraryContext, useLibraryQuery } from '@sd/client';
import { LocationIdParamsSchema } from '~/app/route-schemas';
import { Icon } from '~/components';
import { useZodRouteParams } from '~/hooks';

import Explorer from '../Explorer';
import { ExplorerContextProvider } from '../Explorer/Context';
import { useObjectsInfiniteQuery } from '../Explorer/queries';
import { SearchContextProvider } from '../Explorer/Search/Context';
import { useSearchFilters } from '../Explorer/Search/store';
import { createDefaultExplorerSettings, objectOrderingKeysSchema } from '../Explorer/store';
import { DefaultTopBarOptions } from '../Explorer/TopBarOptions';
import { useExplorer, UseExplorerSettings, useExplorerSettings } from '../Explorer/useExplorer';
import { EmptyNotice } from '../Explorer/View';
import { TopBarPortal } from '../TopBar/Portal';

export const Component = () => {
	return (
		<SearchContextProvider>
			<Inner />
		</SearchContextProvider>
	);
};

function Inner() {
	const { id: tagId } = useZodRouteParams(LocationIdParamsSchema);
	const tag = useLibraryQuery(['tags.get', tagId], { suspense: true });

	const explorerSettings = useExplorerSettings({
		settings: useMemo(
			() =>
				createDefaultExplorerSettings<ObjectOrder>({
					order: null
				}),
			[]
		),
		orderingKeys: objectOrderingKeysSchema
	});

	const { items, count, loadMore, query } = useItems({
		tag: tag.data!,
		settings: explorerSettings
	});

	const explorer = useExplorer({
		items,
		count,
		loadMore,
		settings: explorerSettings,
		...(tag.data && {
			parent: { type: 'Tag', tag: tag.data }
		})
	});

	return (
		<ExplorerContextProvider explorer={explorer}>
			<TopBarPortal
				left={
					<div className="flex flex-row items-center gap-2">
						<div
							className="h-[14px] w-[14px] shrink-0 rounded-full"
							style={{ backgroundColor: tag?.data?.color || '#efefef' }}
						/>
						<span className="truncate text-sm font-medium">{tag?.data?.name}</span>
					</div>
				}
				right={<DefaultTopBarOptions />}
			/>
			<Explorer
				showFilterBar
				emptyNotice={
					<EmptyNotice
						loading={query.isFetching}
						icon={<Icon name="Tags" size={128} />}
						message="No items assigned to this tag."
					/>
				}
			/>
		</ExplorerContextProvider>
	);
}

function useItems({ tag, settings }: { tag: Tag; settings: UseExplorerSettings<ObjectOrder> }) {
	const { library } = useLibraryContext();

	const explorerSettings = settings.useSettingsSnapshot();

	const fixedFilters = useMemo(
		() => [
			{ object: { tags: { in: [tag.id] } } },
			...(explorerSettings.layoutMode === 'media'
				? [{ object: { kind: { in: [ObjectKindEnum.Image, ObjectKindEnum.Video] } } }]
				: [])
		],
		[tag.id, explorerSettings.layoutMode]
	);

	const filters = useSearchFilters('objects', fixedFilters);

	const count = useLibraryQuery(['search.objectsCount', { filters }]);

	const query = useObjectsInfiniteQuery({
		library,
		arg: { take: 100, filters },
		settings
	});

	const items = useMemo(() => query.data?.pages?.flatMap((d) => d.items) ?? null, [query.data]);

	const loadMore = useCallback(() => {
		if (query.hasNextPage && !query.isFetchingNextPage) {
			query.fetchNextPage.call(undefined);
		}
	}, [query.hasNextPage, query.isFetchingNextPage, query.fetchNextPage]);

	return { query, items, loadMore, count: count.data };
}
