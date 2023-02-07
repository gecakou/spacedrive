import { Text, View } from 'react-native';
import { ExplorerItem, isVideoExt } from '@sd/client';
import Layout from '~/constants/Layout';
import { getExplorerStore } from '~/stores/explorerStore';
import tw, { twStyle } from '../../lib/tailwind';
import FileThumb from './FileThumb';

type FileItemProps = {
	data: ExplorerItem;
};

const FileItem = ({ data }: FileItemProps) => {
	const { item } = data;

	const isVid = isVideoExt(item.extension || '');

	const gridItemSize = Layout.window.width / getExplorerStore().gridNumColumns;

	return (
		<View
			style={twStyle('items-center', {
				width: gridItemSize,
				height: gridItemSize
			})}
		>
			<FileThumb
				data={data}
				kind={data.item.extension === 'zip' ? 'zip' : isVid ? 'video' : 'other'}
			/>
			{item.extension && isVid && (
				<View style={tw`absolute bottom-8 right-5 rounded bg-black/70 py-0.5 px-1 opacity-70`}>
					<Text style={tw`text-[9px] font-semibold uppercase text-white`}>{item.extension}</Text>
				</View>
			)}
			<View style={tw`mt-1 px-1.5 py-[1px]`}>
				<Text numberOfLines={1} style={tw`text-center text-xs font-medium text-white`}>
					{item?.name}
					{item?.extension && `.${item.extension}`}
				</Text>
			</View>
		</View>
	);
};

export default FileItem;
