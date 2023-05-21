import clsx from 'clsx';
import { ReactNode, useEffect, useMemo } from 'react';
import { useKey } from 'rooks';
import { ExplorerItem, useLibrarySubscription } from '@sd/client';
import { dialogManager } from '@sd/ui';
import { getExplorerStore, useExplorerStore } from '~/hooks';
import DeleteDialog from '../Explorer/File/DeleteDialog';
import ExplorerContextMenu from './ContextMenu';
import { Inspector } from './Inspector';
import View from './View';
import { useExplorerSearchParams } from './util';

interface Props {
	// TODO: not using data since context isn't actually used
	// and it's not exactly compatible with search
	// data?: ExplorerData;
	items?: ExplorerItem[];
	onLoadMore?(): void;
	hasNextPage?: boolean;
	isFetchingNextPage?: boolean;
	viewClassName?: string;
	children?: ReactNode;
	inspectorClassName?: string;
	explorerClassName?: string;
	listViewHeadersClassName?: string;
	scrollRef?: React.RefObject<HTMLDivElement>;
}

export default function Explorer(props: Props) {
	const { selectedRowIndex, ...expStore } = useExplorerStore();
	const [{ path }] = useExplorerSearchParams();

	useLibrarySubscription(['jobs.newThumbnail'], {
		onStarted: () => {
			console.log('Started RSPC subscription new thumbnail');
		},
		onError: (err) => {
			console.error('Error in RSPC subscription new thumbnail', err);
		},
		onData: (cas_id) => {
			console.log({ cas_id });
			expStore.addNewThumbnail(cas_id);
		}
	});

	useEffect(() => {
		getExplorerStore().selectedRowIndex = null;
	}, [path]);

	const selectedItem = useMemo(() => {
		if (selectedRowIndex === null) return null;

		return props.items?.[selectedRowIndex] ?? null;
	}, [selectedRowIndex, props.items]);

	useKey('Space', (e) => {
		e.preventDefault();

		if (selectedItem) getExplorerStore().quickViewObject = selectedItem;
	});

	useKey('Delete', (e) => {
		e.preventDefault();

		if (!selectedItem) return;

		dialogManager.create((dp) => (
			<DeleteDialog
				{...dp}
				location_id={selectedItem.item.location_id}
				path_id={selectedItem.item.id}
			/>
		));
	});

	return (
		<div className="flex h-screen w-full flex-col bg-app">
			<div className="flex flex-1">
				<div className={clsx('flex-1 overflow-hidden', props.explorerClassName)}>
					{props.children}
					<ExplorerContextMenu>
						{props.items && (
							<View
								scrollRef={props.scrollRef}
								data={props.items}
								onLoadMore={props.onLoadMore}
								hasNextPage={props.hasNextPage}
								listViewHeadersClassName={props.listViewHeadersClassName}
								isFetchingNextPage={props.isFetchingNextPage}
								viewClassName={props.viewClassName}
							/>
						)}
					</ExplorerContextMenu>
				</div>
				{expStore.showInspector && (
					<div className="w-[260px] shrink-0">
						<Inspector className={props.inspectorClassName} data={selectedItem} />
					</div>
				)}
			</div>
		</div>
	);
}
