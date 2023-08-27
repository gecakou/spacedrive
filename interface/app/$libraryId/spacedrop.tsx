import { Broadcast } from 'phosphor-react';
import { Suspense, memo, useDeferredValue, useMemo } from 'react';
import { type NonIndexedPathOrdering, useDiscoveredPeers } from '@sd/client';
import { type PathParams, PathParamsSchema } from '~/app/route-schemas';
import { useOperatingSystem, useZodSearchParams } from '~/hooks';
import Explorer from './Explorer';
import { ExplorerContextProvider } from './Explorer/Context';
import { DefaultTopBarOptions } from './Explorer/TopBarOptions';
import { createDefaultExplorerSettings, nonIndexedPathOrderingSchema } from './Explorer/store';
import { useExplorer, useExplorerSettings } from './Explorer/useExplorer';
import { TopBarPortal } from './TopBar/Portal';

const Spacedrop = memo((props: { args: PathParams }) => {
	const os = useOperatingSystem();
	const discoveredPeers = useDiscoveredPeers();

	const peers = useMemo(() => Array.from(discoveredPeers.values()), [discoveredPeers]);

	const explorerSettings = useExplorerSettings({
		settings: useMemo(
			() =>
				createDefaultExplorerSettings<NonIndexedPathOrdering>({
					order: {
						field: 'name',
						value: 'Asc'
					}
				}),
			[]
		),
		orderingKeys: nonIndexedPathOrderingSchema
	});

	const explorer = useExplorer({
		items: peers.map((peer) => ({
			type: 'SpacedropPeer',
			has_local_thumbnail: false,
			thumbnail_key: null,
			item: peer
		})),
		settings: explorerSettings
	});

	return (
		<ExplorerContextProvider explorer={explorer}>
			<TopBarPortal
				left={
					<div className="flex items-center gap-2">
						<Broadcast className="mt-[-1px] h-[22px] w-[22px]" />
						<span className="truncate text-sm font-medium">Spacedrop</span>
					</div>
				}
				right={<DefaultTopBarOptions />}
				noSearch={true}
			/>
			<Explorer />
		</ExplorerContextProvider>
	);
});

export const Component = () => {
	const [pathParams] = useZodSearchParams(PathParamsSchema);
	const path = useDeferredValue(pathParams);

	return (
		<Suspense>
			<Spacedrop args={path} />
		</Suspense>
	);
};
