import { Suspense, memo, useDeferredValue, useMemo } from 'react';
import { getExplorerItemData, useLibraryQuery } from '@sd/client';
import { Tooltip } from '@sd/ui';
import { PathParams, PathParamsSchema } from '~/app/route-schemas';
import { useOperatingSystem, useZodSearchParams } from '~/hooks';
import Explorer from './Explorer';
import { ExplorerContext } from './Explorer/Context';
import { DefaultTopBarOptions } from './Explorer/TopBarOptions';
import { getExplorerStore, useExplorerStore } from './Explorer/store';
import { useExplorerOrder } from './Explorer/util';
import { TopBarPortal } from './TopBar/Portal';
import { AddLocationButton } from './settings/library/locations/AddLocationButton';

const EphemeralExplorer = memo(({ args: { path } }: { args: PathParams }) => {
	const os = useOperatingSystem();
	const explorerStore = useExplorerStore();

	const query = useLibraryQuery(
		[
			'search.ephemeral-paths',
			{
				path: path ?? (os === 'windows' ? 'C:\\' : '/'),
				withHiddenFiles: true,
				order: useExplorerOrder()
			}
		],
		{
			enabled: !!path,
			onSuccess: () => getExplorerStore().resetNewThumbnails()
		}
	);

	const items =
		useMemo(() => {
			const items = query.data?.entries;
			if (explorerStore.layoutMode !== 'media') return items;

			return items?.filter((item) => {
				const { kind } = getExplorerItemData(item);
				return kind === 'Video' || kind === 'Image';
			});
		}, [query.data, explorerStore.layoutMode]) ?? [];

	return (
		<ExplorerContext.Provider value={{}}>
			<TopBarPortal
				left={
					<Tooltip
						label="Add path as an indexed location"
						className="w-max min-w-0 shrink"
					>
						<AddLocationButton path={path} />
					</Tooltip>
				}
				right={<DefaultTopBarOptions />}
				noSearch={true}
			/>
			<Explorer items={items} />
		</ExplorerContext.Provider>
	);
});

export const Component = () => {
	const [searchParams] = useZodSearchParams(PathParamsSchema);

	const search = useDeferredValue(searchParams);

	return (
		<Suspense fallback="LOADING FIRST RENDER">
			<EphemeralExplorer args={search} />
		</Suspense>
	);
};
