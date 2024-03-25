import { useNavigation } from '@react-navigation/native';
import { FlashList } from '@shopify/flash-list';
import { UseInfiniteQueryResult } from '@tanstack/react-query';
import { AnimatePresence, MotiView } from 'moti';
import { useState } from 'react';
import { ActivityIndicator, Pressable } from 'react-native';
import { isPath, SearchData, type ExplorerItem } from '@sd/client';
import Layout from '~/constants/Layout';
import { tw } from '~/lib/tailwind';
import { BrowseStackScreenProps } from '~/navigation/tabs/BrowseStack';
import { ExplorerLayoutMode, getExplorerStore, useExplorerStore } from '~/stores/explorerStore';
import { useActionsModalStore } from '~/stores/modalStore';

import ScreenContainer from '../layout/ScreenContainer';
import FileItem from './FileItem';
import FileRow from './FileRow';
import Menu from './Menu';

type ExplorerProps = {
	tabHeight?: boolean;
	items: ExplorerItem[] | null;
	/** Function to fetch next page of items. */
	loadMore: () => void;
	query: UseInfiniteQueryResult<SearchData<ExplorerItem>>;
	count?: number;
};

const Explorer = (props: ExplorerProps) => {
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
		<ScreenContainer tabHeight={props.tabHeight} scrollview={false} style={'gap-0 py-0'}>
			{/* Header */}
			{/* Sort By */}
			{/* <SortByMenu /> */}
			<AnimatePresence>
				{explorerStore.toggleMenu && (
					<MotiView
						from={{ translateY: -70 }}
						animate={{ translateY: 0 }}
						transition={{
							type: 'timing',
							duration: 300,
							repeat: 0,
							repeatReverse: false
						}}
						exit={{ translateY: -70 }}
					>
						<Menu
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
			{/* Items */}
			<FlashList
				key={layoutMode}
				numColumns={layoutMode === 'grid' ? getExplorerStore().gridNumColumns : 1}
				data={props.items ?? []}
				keyExtractor={(item) =>
					item.type === 'NonIndexedPath'
						? item.item.path
						: item.type === 'SpacedropPeer'
							? item.item.name
							: item.item.id.toString()
				}
				renderItem={({ item }) => (
					<Pressable onPress={() => handlePress(item)}>
						{layoutMode === 'grid' ? <FileItem data={item} /> : <FileRow data={item} />}
					</Pressable>
				)}
				contentContainerStyle={tw`px-2 py-5`}
				extraData={layoutMode}
				estimatedItemSize={
					layoutMode === 'grid'
						? Layout.window.width / getExplorerStore().gridNumColumns
						: getExplorerStore().listItemSize
				}
				onEndReached={() => props.loadMore?.()}
				onEndReachedThreshold={0.6}
				ListFooterComponent={props.query.isFetchingNextPage ? <ActivityIndicator /> : null}
			/>
		</ScreenContainer>
	);
};

export default Explorer;
