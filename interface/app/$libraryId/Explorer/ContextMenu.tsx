import { Clipboard, FileX, Image, Plus, Repeat, Share, ShieldCheck } from 'phosphor-react';
import { PropsWithChildren, useMemo } from 'react';
import { useBridgeQuery, useLibraryContext, useLibraryMutation, useLibraryQuery } from '@sd/client';
import { ContextMenu as CM, ModifierKeys } from '@sd/ui';
import { showAlertDialog } from '~/components';
import { getExplorerStore, useExplorerStore, useOperatingSystem } from '~/hooks';
import { usePlatform } from '~/util/Platform';
import { keybindForOs } from '~/util/keybinds';
import { useExplorerSearchParams } from './util';

export const OpenInNativeExplorer = () => {
	const os = useOperatingSystem();
	const keybind = keybindForOs(os);

	const osFileBrowserName = useMemo(() => {
		if (os === 'macOS') {
			return 'Finder';
		} else if (os === 'windows') {
			return 'Explorer';
		} else {
			return 'file manager';
		}
	}, [os]);

	const { openPath } = usePlatform();

	let { locationId } = useExplorerStore();
	if (locationId == null) locationId = -1;

	const { library } = useLibraryContext();
	const { data: node } = useBridgeQuery(['nodeState']);
	const { data: location } = useLibraryQuery(['locations.get', locationId]);
	const [{ path: subPath }] = useExplorerSearchParams();

	// Disable for remote nodes, as opening directories in a remote node is a more complex task
	if (!(openPath && location?.path && node?.id && library.config.node_id === node.id))
		return null;
	const path = location.path + (subPath ? subPath : '');

	return (
		<>
			<CM.Item
				label={`Open in ${osFileBrowserName}`}
				keybind={keybind([ModifierKeys.Control], ['Y'])}
				onClick={() => openPath(path)}
			/>

			<CM.Separator />
		</>
	);
};

export default (props: PropsWithChildren) => {
	const os = useOperatingSystem();
	const keybind = keybindForOs(os);
	const [{ path: currentPath }] = useExplorerSearchParams();
	const { locationId, cutCopyState } = useExplorerStore();

	const generateThumbsForLocation = useLibraryMutation('jobs.generateThumbsForLocation');
	const objectValidator = useLibraryMutation('jobs.objectValidator');
	const rescanLocation = useLibraryMutation('locations.fullRescan');
	const copyFiles = useLibraryMutation('files.copyFiles');
	const cutFiles = useLibraryMutation('files.cutFiles');

	return (
		<CM.Root trigger={props.children}>
			<OpenInNativeExplorer />

			<CM.Item
				label="Share"
				icon={Share}
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

			<CM.Separator />

			{locationId && (
				<>
					<CM.Item
						onClick={async () => {
							try {
								await rescanLocation.mutateAsync(locationId);
							} catch (error) {
								showAlertDialog({
									title: 'Error',
									value: `Failed to re-index location, due to an error: ${error}`
								});
							}
						}}
						label="Re-index"
						icon={Repeat}
					/>

					<CM.Item
						label="Paste"
						keybind={keybind([ModifierKeys.Control], ['V'])}
						hidden={!cutCopyState.active}
						onClick={async () => {
							const path = currentPath ?? '/';
							const { actionType, sourcePathId, sourceParentPath, sourceLocationId } =
								cutCopyState;
							const sameLocation =
								sourceLocationId === locationId && sourceParentPath === path;
							try {
								if (actionType == 'Copy') {
									await copyFiles.mutateAsync({
										source_location_id: sourceLocationId,
										sources_file_path_ids: [sourcePathId],
										target_location_id: locationId,
										target_location_relative_directory_path: path,
										target_file_name_suffix: sameLocation ? ' copy' : null
									});
								} else if (sameLocation) {
									showAlertDialog({
										title: 'Error',
										value: `File already exists in this location`
									});
								} else {
									await cutFiles.mutateAsync({
										source_location_id: sourceLocationId,
										sources_file_path_ids: [sourcePathId],
										target_location_id: locationId,
										target_location_relative_directory_path: path
									});
								}
							} catch (error) {
								showAlertDialog({
									title: 'Error',
									value: `Failed to ${actionType.toLowerCase()} file, due to an error: ${error}`
								});
							}
						}}
						icon={Clipboard}
					/>
				</>
			)}

			<CM.Item
				label="Deselect"
				hidden={!cutCopyState.active}
				onClick={() => {
					getExplorerStore().cutCopyState = {
						...cutCopyState,
						active: false
					};
				}}
				icon={FileX}
			/>

			{locationId && (
				<CM.SubMenu label="More actions..." icon={Plus}>
					<CM.Item
						onClick={async () => {
							try {
								await generateThumbsForLocation.mutateAsync({
									id: locationId,
									path: currentPath ?? '/'
								});
							} catch (error) {
								showAlertDialog({
									title: 'Error',
									value: `Failed to generate thumbanails, due to an error: ${error}`
								});
							}
						}}
						label="Regen Thumbnails"
						icon={Image}
					/>

					<CM.Item
						onClick={async () => {
							try {
								objectValidator.mutateAsync({
									id: locationId,
									path: currentPath ?? '/'
								});
							} catch (error) {
								showAlertDialog({
									title: 'Error',
									value: `Failed to generate checksum, due to an error: ${error}`
								});
							}
						}}
						label="Generate Checksums"
						icon={ShieldCheck}
					/>
				</CM.SubMenu>
			)}
		</CM.Root>
	);
};
