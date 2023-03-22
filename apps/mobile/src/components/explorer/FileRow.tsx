import React from 'react';
import { Text, View } from 'react-native';
import { ExplorerItem } from '@sd/client';
import { tw, twStyle } from '~/lib/tailwind';
import { getExplorerStore } from '~/stores/explorerStore';
import FileThumb from './FileThumb';

type FileRowProps = {
	data: ExplorerItem;
};

const FileRow = ({ data }: FileRowProps) => {
	const { item } = data;

	return (
		<View
			style={twStyle('flex flex-row items-center px-3', {
				height: getExplorerStore().listItemSize
			})}
		>
			<FileThumb data={data} size={0.6} />
			<View style={tw`ml-3`}>
				<Text numberOfLines={1} style={tw`text-ink-dull text-center text-xs font-medium`}>
					{item?.name}
					{item?.extension && `.${item.extension}`}
				</Text>
			</View>
		</View>
	);
};

export default FileRow;
