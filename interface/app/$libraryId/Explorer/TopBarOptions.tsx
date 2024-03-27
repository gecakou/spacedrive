import {
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
import { ExplorerLayout, useSelector } from '@sd/client';
import { toast } from '@sd/ui';
import { useKeyMatcher, useLocale } from '~/hooks';

import { KeyManager } from '../KeyManager';
import { Spacedrop, SpacedropButton } from '../Spacedrop';
import TopBarOptions, { ToolOption, TOP_BAR_ICON_STYLE } from '../TopBar/TopBarOptions';
import { useExplorerContext } from './Context';
import OptionsPanel from './OptionsPanel';
import { explorerStore } from './store';

const layoutIcons: Record<ExplorerLayout, Icon> = {
	grid: SquaresFour,
	list: Rows,
	media: MonitorPlay
};

export const useExplorerTopBarOptions = () => {
	const [showInspector, tagAssignMode] = useSelector(explorerStore, (s) => [
		s.showInspector,
		s.tagAssignMode
	]);
	const explorer = useExplorerContext();
	const controlIcon = useKeyMatcher('Meta').icon;
	const settings = explorer.useSettingsSnapshot();

	const { t } = useLocale();

	const viewOptions = useMemo(
		() =>
			(Object.keys(explorer.layouts) as ExplorerLayout[]).reduce(
				(layouts, layout, i) => {
					if (!explorer.layouts[layout]) return layouts;

					const Icon = layoutIcons[layout];

					const option = {
						layout,
						toolTipLabel: t(`${layout} View`),
						icon: <Icon className={TOP_BAR_ICON_STYLE} />,
						keybinds: [controlIcon, (i + 1).toString()],
						topBarActive:
							!explorer.isLoadingPreferences && settings.layoutMode === layout,
						onClick: () => (explorer.settingsStore.layoutMode = layout),
						showAtResolution: 'sm:flex'
					} satisfies ToolOption & { layout: ExplorerLayout };

					return [...layouts, option];
				},
				[] as (ToolOption & { layout: ExplorerLayout })[]
			),
		[
			controlIcon,
			explorer.isLoadingPreferences,
			explorer.layouts,
			explorer.settingsStore,
			settings.layoutMode,
			t
		]
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
				explorerStore.showInspector = !showInspector;
			},
			icon: (
				<SidebarSimple
					weight={showInspector ? 'fill' : 'regular'}
					className={clsx(TOP_BAR_ICON_STYLE, '-scale-x-100')}
				/>
			),
			individual: true,
			showAtResolution: 'xl:flex',
			topBarActive: showInspector
		}
	];

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
			toolTipLabel: 'Spacedrop',
			icon: ({ triggerOpen }) => <SpacedropButton triggerOpen={triggerOpen} />,
			popOverComponent: ({ triggerClose }) => <Spacedrop triggerClose={triggerClose} />,
			individual: true,
			showAtResolution: 'xl:flex'
		},
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
				<Tag weight={tagAssignMode ? 'fill' : 'regular'} className={TOP_BAR_ICON_STYLE} />
			),
			// TODO: Assign tag mode is not yet implemented!
			// onClick: () => (explorerStore.tagAssignMode = !explorerStore.tagAssignMode),
			onClick: () => toast.info('Coming soon!'),
			topBarActive: tagAssignMode,
			individual: true,
			showAtResolution: 'xl:flex'
		}
	] satisfies ToolOption[];

	return {
		viewOptions,
		controlOptions,
		toolOptions
	};
};

export const DefaultTopBarOptions = (props: { options?: ToolOption[] }) => {
	const options = useExplorerTopBarOptions();

	return (
		<TopBarOptions
			options={[
				options.viewOptions,
				[...options.toolOptions, ...(props.options ?? [])],
				options.controlOptions
			]}
		/>
	);
};
