import { Plus } from 'phosphor-react';
import { ExplorerItem } from '@sd/client';
import { ContextMenu } from '@sd/ui';
import { FilePathItems, ObjectItems, SharedItems } from '..';
import { useLibraryQuery } from '../../../../../../packages/client/src';

interface Props {
	data: Extract<ExplorerItem, { type: 'Object' }>;
}

export default ({ data }: Props) => {
	const object = data.item;
	const filePath = data.item.file_paths[0];

	const locationIdToPathQuery = useLibraryQuery(['files.locationIdToPath', { location_id: filePath?.location_id || -1 }])
	const absoluteFilePath = locationIdToPathQuery.data ? `${locationIdToPathQuery.data}${filePath.materialized_path}${filePath.name}${filePath.extension ? `.${filePath.extension}` : ''}` : null

	return (
		<>
			{filePath && <FilePathItems.OpenOrDownload filePath={filePath} />}

			<SharedItems.OpenQuickView item={data} />

			<ContextMenu.Separator />

			<SharedItems.Details />

			<ContextMenu.Separator />

			{filePath && <SharedItems.RevealInNativeExplorer filePath={filePath} />}

			<SharedItems.Rename />

			<ContextMenu.Separator />

			<SharedItems.Share />

			{(object || filePath) && <ContextMenu.Separator />}

			{object && <ObjectItems.AssignTag object={object} />}

			{filePath && (
				<ContextMenu.SubMenu label="More actions..." icon={Plus}>
					{absoluteFilePath && <FilePathItems.CopyAsPath absoluteFilePath={absoluteFilePath} />}
					<FilePathItems.Crypto filePath={filePath} />
					<FilePathItems.Compress filePath={filePath} />
					<ObjectItems.ConvertObject filePath={filePath} object={object} />
				</ContextMenu.SubMenu>
			)}

			{filePath && (
				<>
					<ContextMenu.Separator />
					<FilePathItems.Delete filePath={filePath} />
				</>
			)}
		</>
	);
};
