import { MotiView } from 'moti';
import { Pressable, Text, View } from 'react-native';
import { FlatList } from 'react-native-gesture-handler';
import { LinearTransition } from 'react-native-reanimated';
import { Tag, useCache, useLibraryQuery, useNodes } from '@sd/client';
import { tw, twStyle } from '~/lib/tailwind';
import { getSearchStore, useSearchStore } from '~/stores/searchStore';

import Fade from '../layout/Fade';
import SectionTitle from '../layout/SectionTitle';
import VirtualizedListWrapper from '../layout/VirtualizedListWrapper';

export const Tags = () => {
	const tags = useLibraryQuery(['tags.list']);
	useNodes(tags.data?.nodes);
	const tagsData = useCache(tags.data?.items);

	return (
		<MotiView
			layout={LinearTransition.duration(300)}
			from={{ opacity: 0, translateY: 20 }}
			animate={{ opacity: 1, translateY: 0 }}
			transition={{ type: 'timing', duration: 300 }}
			exit={{ opacity: 0 }}
		>
			<SectionTitle
				style={tw`px-6 pb-3`}
				title="Tags"
				sub="What tags would you like to filter by?"
			/>
			<View>
				<Fade color="mobile-screen" width={30} height="100%">
					<VirtualizedListWrapper horizontal>
						<FlatList
							data={tagsData}
							renderItem={({ item }) => (
								<Pressable
									onPress={() => getSearchStore().updateFilters('tags', item.id)}
								>
									<TagFilter tag={item} />
								</Pressable>
							)}
							contentContainerStyle={tw`pl-6`}
							extraData={useSearchStore().filters}
							numColumns={tagsData && Math.ceil(Number(tagsData.length ?? 0) / 2)}
							key={tagsData ? 'tagsSearch' : '_'}
							scrollEnabled={false}
							ItemSeparatorComponent={() => <View style={tw`w-2 h-2`} />}
							keyExtractor={(item) => item.id.toString()}
							showsHorizontalScrollIndicator={false}
							style={tw`flex-row`}
						/>
					</VirtualizedListWrapper>
				</Fade>
			</View>
		</MotiView>
	);
};

interface Props {
	tag: Tag;
}

const TagFilter = ({ tag }: Props) => {
	const isSelected = useSearchStore().isFilterSelected('tags', tag.id);
	return (
		<MotiView
			animate={{
				borderColor: isSelected ? tag.color! : tw.color('app-line/50')
			}}
			style={tw`mr-2 w-auto flex-row items-center gap-2 rounded-md border border-app-line/50 bg-app-box/50 p-2.5`}
		>
			<View
				style={twStyle(`h-5 w-5 rounded-full`, {
					backgroundColor: tag.color!
				})}
			/>
			<Text style={tw`text-sm font-medium text-ink`}>{tag?.name}</Text>
		</MotiView>
	);
};

export default Tags;
