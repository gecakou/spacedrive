import { useMemo } from 'react';
import { View } from 'react-native';
import Animated, { useAnimatedScrollHandler } from 'react-native-reanimated';
import { useDebounce } from 'use-debounce';
import { useLibraryQuery } from '@sd/client';
import { IconName } from '~/components/icons/Icon';
import ScreenContainer from '~/components/layout/ScreenContainer';
import CategoryItem from '~/components/overview/CategoryItem';
import { tw } from '~/lib/tailwind';
import { useSearchStore } from '~/stores/searchStore';
import { ScrollY } from '~/types/shared';

const CategoriesScreen = ({ scrollY }: ScrollY) => {
	const kinds = useLibraryQuery(['library.kindStatistics']);
	const { search } = useSearchStore();
	const [debouncedSearch] = useDebounce(search, 200);
	const filteredKinds = useMemo(
		() =>
			kinds.data?.statistics.filter((kind) =>
				kind.name?.toLowerCase().includes(debouncedSearch.toLowerCase())
			) ?? [],
		[debouncedSearch, kinds]
	);
	const scrollHandler = useAnimatedScrollHandler((e) => {
		scrollY.value = e.contentOffset.y;
	});
	return (
		<ScreenContainer scrollview={false} style={tw`relative px-6 py-0`}>
			<Animated.FlatList
				data={filteredKinds?.sort((a, b) => b.count - a.count).filter((i) => i.kind !== 0)}
				numColumns={3}
				onScroll={scrollHandler}
				contentContainerStyle={tw`py-6`}
				keyExtractor={(item) => item.name}
				scrollEventThrottle={1}
				ItemSeparatorComponent={() => <View style={tw`h-2`} />}
				showsVerticalScrollIndicator={false}
				showsHorizontalScrollIndicator={false}
				renderItem={({ item }) => {
					const { kind, name, count } = item;
					let icon = name as IconName;
					switch (name) {
						case 'Code':
							icon = 'Terminal';
							break;
						case 'Unknown':
							icon = 'Undefined';
							break;
					}
					return (
						<CategoryItem
							style={'mx-1 w-[31.4%]'}
							kind={kind}
							name={name}
							icon={icon}
							items={count}
						/>
					);
				}}
			/>
		</ScreenContainer>
	);
};

export default CategoriesScreen;
