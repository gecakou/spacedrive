import { MagnifyingGlass } from 'phosphor-react-native';
import { useState } from 'react';
import { ActivityIndicator, Pressable, Text, TextInput, View } from 'react-native';
import { useSafeAreaInsets } from 'react-native-safe-area-context';
import { Button } from '~/components/primitive/Button';
import tw from '~/lib/tailwind';
import { RootStackScreenProps } from '~/navigation';

const SearchScreen = ({ navigation }: RootStackScreenProps<'Search'>) => {
	const { top } = useSafeAreaInsets();

	const [loading, setLoading] = useState(false);

	// TODO: Animations!

	return (
		<View style={tw.style('flex-1', { marginTop: top + 10 })}>
			{/* Header */}
			<View style={tw`flex flex-row items-center mx-4`}>
				{/* Search Input */}
				<View style={tw`flex-1 bg-app-overlay border border-app-line rounded h-10 mr-3`}>
					<View style={tw`flex flex-row h-full items-center px-3`}>
						<View style={tw`mr-3`}>
							{loading ? (
								<ActivityIndicator size={'small'} color={'white'} />
							) : (
								<MagnifyingGlass size={20} weight="light" color={tw.color('ink-faint')} />
							)}
						</View>
						<TextInput
							placeholder={'Search'}
							clearButtonMode="never" // can't change the color??
							underlineColorAndroid="transparent"
							placeholderTextColor={tw.color('ink-dull')}
							style={tw`flex-1 text-ink font-medium text-sm`}
							textContentType={'none'}
							autoFocus
							autoCapitalize="none"
							autoCorrect={false}
						/>
					</View>
				</View>
				{/* Cancel Button */}
				<Pressable onPress={() => navigation.goBack()}>
					<Text style={tw`text-accent`}>Cancel</Text>
				</Pressable>
			</View>
			{/* Content */}
			<View style={tw`flex-1 items-center mt-8`}>
				<Button variant="accent" onPress={() => setLoading((v) => !v)}>
					<Text>Toggle loading</Text>
				</Button>
			</View>
		</View>
	);
};

export default SearchScreen;
