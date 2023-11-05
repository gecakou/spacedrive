import {
	ArrowClockwise,
	Icon,
	Key,
	MonitorPlay,
	Rows,
	SidebarSimple,
	SlidersHorizontal,
	SquaresFour,
	Tag
} from '@phosphor-icons/react';
import clsx from 'clsx';
import { useMemo } from 'react';
import { useDocumentEventListener } from 'rooks';
import { ExplorerLayout } from '@sd/client';
import { ModifierKeys, toast } from '@sd/ui';
import { useKeybind, useKeyMatcher, useOperatingSystem } from '~/hooks';

import { useQuickRescan } from '../../../hooks/useQuickRescan';
import { KeyManager } from '../KeyManager';
import TopBarOptions, { ToolOption, TOP_BAR_ICON_STYLE } from '../TopBar/TopBarOptions';
import { useExplorerContext } from './Context';
import OptionsPanel from './OptionsPanel';
import { getExplorerStore, useExplorerStore } from './store';

const layoutIcons: Record<ExplorerLayout, Icon> = {
	grid: SquaresFour,
	list: Rows,
	media: MonitorPlay
};

export const useExplorerTopBarOptions = () => {
	const explorerStore = useExplorerStore();
	const explorer = useExplorerContext();
	const controlIcon = useKeyMatcher('Meta').icon;
	const settings = explorer.useSettingsSnapshot();
	const rescan = useQuickRescan();

	const viewOptions = useMemo(
		() =>
			(Object.keys(explorer.layouts) as ExplorerLayout[]).reduce(
				(layouts, layout, i) => {
					if (!explorer.layouts[layout]) return layouts;

					const Icon = layoutIcons[layout];

					const option = {
						layout,
						toolTipLabel: `${layout} view`,
						icon: <Icon className={TOP_BAR_ICON_STYLE} />,
						keybinds: [controlIcon, (i + 1).toString()],
						topBarActive: settings.layoutMode === layout,
						onClick: () => (explorer.settingsStore.layoutMode = layout),
						showAtResolution: 'sm:flex'
					} satisfies ToolOption & { layout: ExplorerLayout };

					return [...layouts, option];
				},
				[] as (ToolOption & { layout: ExplorerLayout })[]
			),
		[controlIcon, explorer.layouts, explorer.settingsStore, settings.layoutMode]
	);

	const controlOptions: ToolOption[] = [
		{
			toolTipLabel: 'Explorer display',
			icon: <SlidersHorizontal className={TOP_BAR_ICON_STYLE} />,
			popOverComponent: <OptionsPanel />,
			individual: true,
			showAtResolution: 'sm:flex'
		},
		{
			toolTipLabel: 'Show Inspector',
			keybinds: [controlIcon, 'I'],
			onClick: () => {
				getExplorerStore().showInspector = !explorerStore.showInspector;
			},
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

	const { parent } = useExplorerContext();

	const os = useOperatingSystem();

	useKeybind([os === 'macOS' ? ModifierKeys.Meta : ModifierKeys.Control, 'r'], () => rescan());

	useDocumentEventListener('keydown', (e: unknown) => {
		if (!(e instanceof KeyboardEvent)) return;

		const meta = e.metaKey || e.ctrlKey;
		if (!meta) return;

		const layout = viewOptions[Number(e.key) - 1]?.layout;
		if (!layout) return;

		e.stopPropagation();
		explorer.settingsStore.layoutMode = layout;
	});

	const toolOptions = [
		{
			toolTipLabel: 'Key Manager',
			icon: <Key className={TOP_BAR_ICON_STYLE} />,
			popOverComponent: <KeyManager />,
			individual: true,
			showAtResolution: 'xl:flex'
		},
		{
			toolTipLabel: 'Tag Assign Mode',
			icon: (
				<Tag
					weight={explorerStore.tagAssignMode ? 'fill' : 'regular'}
					className={TOP_BAR_ICON_STYLE}
				/>
			),
			// TODO: Assign tag mode is not yet implemented!
			// onClick: () => (getExplorerStore().tagAssignMode = !explorerStore.tagAssignMode),
			onClick: () => toast.info('Coming soon!'),
			topBarActive: explorerStore.tagAssignMode,
			individual: true,
			showAtResolution: 'xl:flex'
		},
		parent?.type === 'Location' && {
			toolTipLabel: 'Reload',
			onClick: rescan,
			icon: <ArrowClockwise className={TOP_BAR_ICON_STYLE} />,
			individual: true,
			showAtResolution: 'xl:flex'
		}
	].filter(Boolean) as ToolOption[];

	return {
		viewOptions,
		controlOptions,
		toolOptions
	};
};

export const DefaultTopBarOptions = () => {
	const options = useExplorerTopBarOptions();

	return (
		<TopBarOptions
			options={[options.viewOptions, options.toolOptions, options.controlOptions]}
		/>
	);
};
