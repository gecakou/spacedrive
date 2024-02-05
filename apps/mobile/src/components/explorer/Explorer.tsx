import { useNavigation } from '@react-navigation/native';
import { FlashList } from '@shopify/flash-list';
import { AnimatePresence, MotiView } from 'moti';
import { MonitorPlay, Rows, SlidersHorizontal, SquaresFour } from 'phosphor-react-native';
import { useState } from 'react';
import { Pressable, View } from 'react-native';
import { isPath, type ExplorerItem } from '@sd/client';
import Layout from '~/constants/Layout';
import { tw } from '~/lib/tailwind';
import { BrowseStackScreenProps } from '~/navigation/tabs/BrowseStack';
import { ExplorerLayoutMode, getExplorerStore, useExplorerStore } from '~/stores/explorerStore';
import { useActionsModalStore } from '~/stores/modalStore';

import FileItem from './FileItem';
import FileRow from './FileRow';

type ExplorerProps = {
	items?: ExplorerItem[];
};

const Explorer = ({ items }: ExplorerProps) => {
	const navigation = useNavigation<BrowseStackScreenProps<'Location'>['navigation']>();
	const explorerStore = useExplorerStore();
	const [layoutMode, setLayoutMode] = useState<ExplorerLayoutMode>(getExplorerStore().layoutMode);

	function changeLayoutMode(kind: ExplorerLayoutMode) {
		// We need to keep layoutMode as a state to make sure flash-list re-renders.
		setLayoutMode(kind);
		getExplorerStore().layoutMode = kind;
	}

	const { modalRef, setData } = useActionsModalStore();

	function handlePress(data: ExplorerItem) {
		if (isPath(data) && data.item.is_dir && data.item.location_id !== null) {
			navigation.push('Location', {
				id: data.item.location_id,
				path: `${data.item.materialized_path}${data.item.name}/`
			});
		} else {
			setData(data);
			modalRef.current?.present();
		}
	}

	return (
		<View style={tw`flex-1 bg-mobile-screen`}>
			{/* Header */}
			<View style={tw`flex flex-row items-center justify-between`}>
				{/* Sort By */}
				{/* <SortByMenu /> */}
				<AnimatePresence>
					{explorerStore.toggleMenu && (
						<MotiView
							from={{ translateY: -70 }}
							animate={{ translateY: 0 }}
							transition={{ type: 'timing', duration: 300 }}
							exit={{ translateY: -70 }}
						>
							<LocationMenu
								changeLayoutMode={(kind: ExplorerLayoutMode) => {
									changeLayoutMode(kind);
								}}
								layoutMode={layoutMode}
							/>
						</MotiView>
					)}
				</AnimatePresence>
				{/* Layout (Grid/List) */}
				{/* {layoutMode === 'grid' ? (
					<Pressable onPress={() => changeLayoutMode('list')}>
						<SquaresFour color={tw.color('ink')} size={23} />
					</Pressable>
				) : (
					<Pressable onPress={() => changeLayoutMode('grid')}>
						<Rows color={tw.color('ink')} size={23} />
					</Pressable>
				)} */}
			</View>
			{/* Items */}
			{items && (
				<FlashList
					key={layoutMode}
					numColumns={layoutMode === 'grid' ? getExplorerStore().gridNumColumns : 1}
					data={items}
					keyExtractor={(item) =>
						item.type === 'NonIndexedPath'
							? item.item.path
							: item.type === 'SpacedropPeer'
							? item.item.name
							: item.item.id.toString()
					}
					renderItem={({ item }) => (
						<Pressable onPress={() => handlePress(item)}>
							{layoutMode === 'grid' ? (
								<FileItem data={item} />
							) : (
								<FileRow data={item} />
							)}
						</Pressable>
					)}
					extraData={layoutMode}
					estimatedItemSize={
						layoutMode === 'grid'
							? Layout.window.width / getExplorerStore().gridNumColumns
							: getExplorerStore().listItemSize
					}
				/>
			)}
		</View>
	);
};

interface LocationMenuProps {
	layoutMode: ExplorerLayoutMode;
	changeLayoutMode: (kind: ExplorerLayoutMode) => void;
}

const LocationMenu = ({ layoutMode, changeLayoutMode }: LocationMenuProps) => {
	return (
		<View
			style={tw`flex-row justify-between w-screen py-4 border-b border-app-line/50 bg-mobile-header px-7`}
		>
			<View style={tw`flex-row gap-3`}>
				<Pressable onPress={() => changeLayoutMode('grid')}>
					<SquaresFour
						color={tw.color(layoutMode === 'grid' ? 'text-accent' : 'text-ink-dull')}
						size={23}
					/>
				</Pressable>
				<Pressable onPress={() => changeLayoutMode('list')}>
					<Rows
						color={tw.color(layoutMode === 'list' ? 'text-accent' : 'text-ink-dull')}
						size={23}
					/>
				</Pressable>
				<Pressable onPress={() => changeLayoutMode('media')}>
					<MonitorPlay
						color={tw.color(layoutMode === 'media' ? 'text-accent' : 'text-ink-dull')}
						size={23}
					/>
				</Pressable>
			</View>
			<SlidersHorizontal style={tw`text-ink-dull`} />
		</View>
	);
};

export default Explorer;
