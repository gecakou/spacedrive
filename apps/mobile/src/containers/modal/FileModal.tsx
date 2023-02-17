import { BottomSheetModal, BottomSheetScrollView } from '@gorhom/bottom-sheet';
import dayjs from 'dayjs';
import { CaretLeft } from 'phosphor-react-native';
import { useRef } from 'react';
import { Button, Pressable, Text, View } from 'react-native';
import { default as FileIcon, default as FileThumb } from '../../components/explorer/FileThumb';
import { Modal } from '../../components/layout/Modal';
import Divider from '../../components/primitive/Divider';
import tw from '../../lib/tailwind';
import { useFileModalStore } from '../../stores/modalStore';

type MetaItemProps = {
	title: string;
	value: string;
};

function MetaItem({ title, value }: MetaItemProps) {
	return (
		<View>
			<Text style={tw`text-sm font-bold text-white`}>{title}</Text>
			<Text style={tw`mt-1 text-sm text-gray-400`}>{value}</Text>
		</View>
	);
}

export const FileModal = () => {
	const { fileRef, data } = useFileModalStore();

	const fileDetailsRef = useRef<BottomSheetModal>(null);

	const item = data.item;

	return (
		<>
			<Modal ref={fileRef} snapPoints={['60%', '90%']}>
				{data && (
					<View style={tw`bg-app flex-1 p-4`}>
						{/* File Icon / Name */}
						<View style={tw`flex flex-row items-center`}>
							<FileIcon data={data} size={1.6} />
							{/* File Name, Details etc. */}
							<View style={tw`ml-2`}>
								<Text style={tw`text-base font-bold text-gray-200`}>{item.name}</Text>
								<View style={tw`mt-2 flex flex-row`}>
									<Text style={tw`text-xs text-gray-400`}>5 MB,</Text>
									<Text style={tw`ml-1 text-xs text-gray-400`}>
										{item.extension.toUpperCase()},
									</Text>
									<Text style={tw`ml-1 text-xs text-gray-400`}>15 Aug</Text>
								</View>
								<Pressable style={tw`mt-2`} onPress={() => fileDetailsRef.current.present()}>
									<Text style={tw`text-accent text-sm`}>More</Text>
								</Pressable>
							</View>
						</View>
						{/* Divider */}
						<Divider style={tw`my-6`} />
						{/* Buttons */}
						<Button onPress={() => fileRef.current.close()} title="Copy" color="white" />
						<Button onPress={() => fileRef.current.close()} title="Move" color="white" />
						<Button onPress={() => fileRef.current.close()} title="Share" color="white" />
						<Button onPress={() => fileRef.current.close()} title="Delete" color="white" />
					</View>
				)}
			</Modal>
			{/* Details Modal */}
			<Modal
				ref={fileDetailsRef}
				enableContentPanningGesture={false}
				enablePanDownToClose={false}
				snapPoints={['70%']}
			>
				{data && (
					<BottomSheetScrollView style={tw`bg-app flex-1 p-4`}>
						{/* Back Button */}
						<Pressable style={tw`ml-4 w-full`} onPress={() => fileDetailsRef.current.close()}>
							<CaretLeft color={tw.color('accent')} size={20} />
						</Pressable>
						{/* File Icon / Name */}
						<View style={tw`items-center`}>
							<FileThumb data={data} size={1.8} />
							<Text style={tw`mt-3 text-base font-bold text-gray-200`}>{item.name}</Text>
						</View>
						{/* Details */}
						<Divider style={tw`mt-6 mb-4`} />
						<>
							{/* Temp, we need cas id */}
							{item.id && <MetaItem title="Unique Content ID" value={'555555555'} />}
							<Divider style={tw`my-4`} />
							<MetaItem title="URI" value={`/Users/utku/Somewhere/vite.config.js`} />
							<Divider style={tw`my-4`} />
							<MetaItem
								title="Date Created"
								value={dayjs(item.date_created).format('MMMM Do yyyy, h:mm:ss aaa')}
							/>
							<Divider style={tw`my-4`} />
							<MetaItem
								title="Date Indexed"
								value={dayjs(item.date_indexed).format('MMMM Do yyyy, h:mm:ss aaa')}
							/>
						</>
					</BottomSheetScrollView>
				)}
			</Modal>
		</>
	);
};
