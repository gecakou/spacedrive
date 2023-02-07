import { Text, View } from 'react-native';
import { ExplorerItem, isVideoExt } from '@sd/client';
import Layout from '~/constants/Layout';
import { getExplorerStore } from '~/stores/explorerStore';
import tw from '../../lib/tailwind';
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
			style={tw.style('items-center', {
				width: gridItemSize,
				height: gridItemSize
			})}
		>
			<FileThumb
				data={data}
				kind={data.item.extension === 'zip' ? 'zip' : isVid ? 'video' : 'other'}
			/>
			{item.extension && isVid && (
				<View style={tw`absolute bottom-8 opacity-70 right-5 py-0.5 px-1 bg-black/70 rounded`}>
					<Text style={tw`text-[9px] text-white uppercase font-semibold`}>{item.extension}</Text>
				</View>
			)}
			<View style={tw`px-1.5 py-[1px] mt-1`}>
				<Text numberOfLines={1} style={tw`text-xs font-medium text-center text-white`}>
					{item?.name}
					{item?.extension && `.${item.extension}`}
				</Text>
			</View>
		</View>
	);
};

export default FileItem;
