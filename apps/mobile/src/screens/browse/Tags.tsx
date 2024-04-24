import { useNavigation } from '@react-navigation/native';
import { Plus } from 'phosphor-react-native';
import { useRef } from 'react';
import { Pressable, View } from 'react-native';
import { FlatList } from 'react-native-gesture-handler';
import { useCache, useLibraryQuery, useNodes } from '@sd/client';
import Empty from '~/components/layout/Empty';
import { ModalRef } from '~/components/layout/Modal';
import ScreenContainer from '~/components/layout/ScreenContainer';
import CreateTagModal from '~/components/modal/tag/CreateTagModal';
import { TagItem } from '~/components/tags/TagItem';
import { tw, twStyle } from '~/lib/tailwind';
import { BrowseStackScreenProps } from '~/navigation/tabs/BrowseStack';

interface Props {
	viewStyle?: 'grid' | 'list';
}

export default function TagsScreen({ viewStyle = 'list' }: Props) {
	const navigation = useNavigation<BrowseStackScreenProps<'Browse'>['navigation']>();
	const modalRef = useRef<ModalRef>(null);

	const tags = useLibraryQuery(['tags.list']);
	useNodes(tags.data?.nodes);
	const tagData = useCache(tags.data?.items);

	return (
		<ScreenContainer scrollview={false} style={tw`relative px-6 py-0`}>
			<Pressable
				style={tw`absolute bottom-7 right-7 z-10 flex h-12 w-12 items-center justify-center rounded-full bg-accent`}
				testID="create-tag-modal"
				onPress={() => {
					modalRef.current?.present();
				}}
			>
				<Plus size={20} weight="bold" style={tw`text-ink`} />
			</Pressable>
				<FlatList
					data={tagData}
					renderItem={({ item }) => (
						<TagItem
							viewStyle={viewStyle}
							tag={item}
							onPress={() => {
								navigation.navigate('BrowseStack', {
									screen: 'Tag',
									params: { id: item.id, color: item.color! }
								});
							}}
						/>
					)}
					ListEmptyComponent={
						<Empty
							icon="Tags"
							style={'border-0'}
							textSize="text-md"
							iconSize={84}
							description="You have not created any tags"
						/>
					}
					horizontal={false}
					numColumns={viewStyle === 'grid' ? 3 : 1}
					keyExtractor={(item) => item.id.toString()}
					showsHorizontalScrollIndicator={false}
					ItemSeparatorComponent={() => <View style={tw`h-2`} />}
					contentContainerStyle={twStyle(
						`py-6`,
						tagData.length === 0 && 'h-full items-center justify-center'
					)}
				/>
			<CreateTagModal ref={modalRef} />
		</ScreenContainer>
	);
}
