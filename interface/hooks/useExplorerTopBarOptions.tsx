import clsx from 'clsx';
import {
	ArrowClockwise,
	Columns,
	Key,
	MonitorPlay,
	Rows,
	SidebarSimple,
	SlidersHorizontal,
	SquaresFour,
	Tag
} from 'phosphor-react';
import { useEffect, useRef } from 'react';
import { useRspcLibraryContext } from '@sd/client';
import OptionsPanel from '~/app/$libraryId/Explorer/OptionsPanel';
import { TOP_BAR_ICON_STYLE, ToolOption } from '~/app/$libraryId/TopBar/TopBarOptions';
// import { KeyManager } from '../app/$libraryId/KeyManager';
import { getExplorerStore, useExplorerStore } from './useExplorerStore';

export const useExplorerTopBarOptions = () => {
	const explorerStore = useExplorerStore();

	const explorerViewOptions: ToolOption[] = [
		{
			toolTipLabel: 'Grid view',
			icon: <SquaresFour className={TOP_BAR_ICON_STYLE} />,
			topBarActive: explorerStore.layoutMode === 'grid',
			onClick: () => (getExplorerStore().layoutMode = 'grid'),
			showAtResolution: 'sm:flex'
		},
		{
			toolTipLabel: 'List view',
			icon: <Rows className={TOP_BAR_ICON_STYLE} />,
			topBarActive: explorerStore.layoutMode === 'rows',
			onClick: () => (getExplorerStore().layoutMode = 'rows'),
			showAtResolution: 'sm:flex'
		},
		{
			toolTipLabel: 'Columns view',
			icon: <Columns className={TOP_BAR_ICON_STYLE} />,
			topBarActive: explorerStore.layoutMode === 'columns',
			onClick: () => (getExplorerStore().layoutMode = 'columns'),
			showAtResolution: 'sm:flex'
		},
		{
			toolTipLabel: 'Media view',
			icon: <MonitorPlay className={TOP_BAR_ICON_STYLE} />,
			topBarActive: explorerStore.layoutMode === 'media',
			onClick: () => (getExplorerStore().layoutMode = 'media'),
			showAtResolution: 'sm:flex'
		}
	];

	const explorerControlOptions: ToolOption[] = [
		{
			toolTipLabel: 'Explorer display',
			icon: <SlidersHorizontal className={TOP_BAR_ICON_STYLE} />,
			popOverComponent: <OptionsPanel />,
			individual: true,
			showAtResolution: 'xl:flex'
		},
		{
			toolTipLabel: 'Show Inspector',
			onClick: () => (getExplorerStore().showInspector = !explorerStore.showInspector),
			icon: (
				<SidebarSimple
					weight={explorerStore.showInspector ? 'fill' : 'regular'}
					className={clsx(TOP_BAR_ICON_STYLE, 'scale-x-[-1]')}
				/>
			),
			individual: true,
			showAtResolution: 'xl:flex',
			topBarActive: explorerStore.showInspector
		}
	];

	// subscription so that we can cancel it if in progress
	const quickRescanSubscription = useRef<() => void | undefined>();

	// gotta clean up any rescan subscriptions if the exist
	useEffect(() => () => quickRescanSubscription.current?.(), []);

	const { client } = useRspcLibraryContext();

	const explorerToolOptions: ToolOption[] = [
		// {
		// 	toolTipLabel: 'Key Manager',
		// 	icon: <Key className={TOP_BAR_ICON_STYLE} />,
		// 	popOverComponent: <KeyManager />,
		// 	individual: true,
		// 	showAtResolution: 'xl:flex'
		// },
		{
			toolTipLabel: 'Tag Assign Mode',
			icon: (
				<Tag
					weight={explorerStore.tagAssignMode ? 'fill' : 'regular'}
					className={TOP_BAR_ICON_STYLE}
				/>
			),
			onClick: () => (getExplorerStore().tagAssignMode = !explorerStore.tagAssignMode),
			topBarActive: explorerStore.tagAssignMode,
			individual: true,
			showAtResolution: 'xl:flex'
		},
		{
			toolTipLabel: 'Reload',
			onClick: () => {
				if (explorerStore.locationId) {
					quickRescanSubscription.current?.();
					quickRescanSubscription.current = client.addSubscription(
						[
							'locations.quickRescan',
							{
								location_id: explorerStore.locationId,
								sub_path: ''
							}
						],
						{ onData() {} }
					);
				}
			},
			icon: <ArrowClockwise className={TOP_BAR_ICON_STYLE} />,
			individual: true,
			showAtResolution: 'xl:flex'
		}
	];

	return { explorerViewOptions, explorerControlOptions, explorerToolOptions };
};
