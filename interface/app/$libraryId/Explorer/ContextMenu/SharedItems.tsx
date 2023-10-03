import { FileX, Share as ShareIcon } from '@phosphor-icons/react';
import { useMemo } from 'react';
import { useLibraryContext, useLibraryMutation } from '@sd/client';
import { ContextMenu, ModifierKeys, toast } from '@sd/ui';
import { Menu } from '~/components/Menu';
import { useKeybindFactory } from '~/hooks/useKeybindFactory';
import { isNonEmpty } from '~/util';
import { usePlatform, type Platform } from '~/util/Platform';

import { useExplorerContext } from '../Context';
import { getQuickPreviewStore } from '../QuickPreview/store';
import { RevealInNativeExplorerBase } from '../RevealInNativeExplorer';
import { getExplorerStore, useExplorerStore } from '../store';
import { useExplorerViewContext } from '../ViewContext';
import { Conditional, ConditionalItem } from './ConditionalItem';
import { useContextMenuContext } from './context';
import OpenWith from './OpenWith';

export const OpenOrDownload = new ConditionalItem({
	useCondition: () => {
		const { selectedFilePaths, selectedEphemeralPaths } = useContextMenuContext();
		const { openFilePaths, openEphemeralFiles } = usePlatform();

		if (
			!openFilePaths ||
			!openEphemeralFiles ||
			(!isNonEmpty(selectedFilePaths) && !isNonEmpty(selectedEphemeralPaths))
		)
			return null;

		return { openFilePaths, openEphemeralFiles, selectedFilePaths, selectedEphemeralPaths };
	},
	Component: ({
		openFilePaths,
		openEphemeralFiles,
		selectedFilePaths,
		selectedEphemeralPaths
	}) => {
		const keybind = useKeybindFactory();
		const { platform } = usePlatform();
		const updateAccessTime = useLibraryMutation('files.updateAccessTime');

		const { library } = useLibraryContext();

		if (platform === 'web') return <Menu.Item label="Download" />;
		else
			return (
				<>
					<Menu.Item
						label="Open"
						keybind={keybind([ModifierKeys.Control], ['O'])}
						onClick={async () => {
							try {
								if (selectedFilePaths.length > 0) {
									updateAccessTime
										.mutateAsync(
											selectedFilePaths
												.map((p) => p.object_id!)
												.filter(Boolean)
										)
										.catch(console.error);

									await openFilePaths(
										library.uuid,
										selectedFilePaths.map((p) => p.id)
									);
								} else if (selectedEphemeralPaths.length > 0) {
									await openEphemeralFiles(
										selectedEphemeralPaths.map((p) => p.path)
									);
								}
							} catch (error) {
								toast.error({
									title: `Failed to open file`,
									body: `Error: ${error}.`
								});
							}
						}}
					/>
					<Conditional items={[OpenWith]} />
				</>
			);
	}
});

export const OpenQuickView = () => {
	const keybind = useKeybindFactory();

	return (
		<ContextMenu.Item
			label="Quick view"
			keybind={keybind([], [' '])}
			onClick={() => (getQuickPreviewStore().open = true)}
		/>
	);
};

export const Details = new ConditionalItem({
	useCondition: () => {
		const { showInspector } = useExplorerStore();
		if (showInspector) return null;

		return {};
	},
	Component: () => {
		const keybind = useKeybindFactory();

		return (
			<ContextMenu.Item
				label="Details"
				keybind={keybind([ModifierKeys.Control], ['I'])}
				// icon={Sidebar}
				onClick={() => (getExplorerStore().showInspector = true)}
			/>
		);
	}
});

export const Rename = new ConditionalItem({
	useCondition: () => {
		const { selectedItems } = useContextMenuContext();

		const settings = useExplorerContext().useSettingsSnapshot();

		if (
			settings.layoutMode === 'media' ||
			selectedItems.length > 1 ||
			selectedItems.some((item) => item.type === 'NonIndexedPath')
		)
			return null;

		return {};
	},
	Component: () => {
		const explorerView = useExplorerViewContext();
		const keybind = useKeybindFactory();

		return (
			<ContextMenu.Item
				label="Rename"
				keybind={keybind([], ['Enter'])}
				onClick={() => explorerView.setIsRenaming(true)}
			/>
		);
	}
});

export const RevealInNativeExplorer = new ConditionalItem({
	useCondition: () => {
		const { selectedItems } = useContextMenuContext();

		const items = useMemo(() => {
			const array: Parameters<NonNullable<Platform['revealItems']>>[1] = [];

			for (const item of selectedItems) {
				switch (item.type) {
					case 'Path': {
						array.push({
							FilePath: { id: item.item.id }
						});
						break;
					}
					case 'Object': {
						// this isn't good but it's the current behaviour
						const filePath = item.item.file_paths[0];
						if (filePath)
							array.push({
								FilePath: {
									id: filePath.id
								}
							});
						else return [];
						break;
					}
					case 'Location': {
						array.push({
							Location: {
								id: item.item.id
							}
						});
						break;
					}
					case 'NonIndexedPath': {
						array.push({
							Ephemeral: {
								path: item.item.path
							}
						});
						break;
					}
				}
			}

			return array;
		}, [selectedItems]);

		if (!isNonEmpty(items)) return null;

		return { items };
	},
	Component: ({ items }) => <RevealInNativeExplorerBase items={items} />
});

export const Deselect = new ConditionalItem({
	useCondition: () => {
		const { cutCopyState } = useExplorerStore();

		if (cutCopyState.type === 'Idle') return null;

		return {};
	},
	Component: () => (
		<ContextMenu.Item
			label="Deselect"
			icon={FileX}
			onClick={() => {
				getExplorerStore().cutCopyState = {
					type: 'Idle'
				};
			}}
		/>
	)
});

export const Share = () => {
	return (
		<>
			<Menu.Item
				label="Share"
				icon={ShareIcon}
				onClick={(e) => {
					e.preventDefault();

					navigator.share?.({
						title: 'Spacedrive',
						text: 'Check out this cool app',
						url: 'https://spacedrive.com'
					});
				}}
				disabled
			/>
		</>
	);
};
