import { CompositeScreenProps } from '@react-navigation/native';
import { createStackNavigator, StackScreenProps } from '@react-navigation/stack';
import Header from '~/components/header/Header';
import { tw } from '~/lib/tailwind';
import BrowseScreen from '~/screens/Browse';

import { SharedScreens, SharedScreensParamList } from '../SharedScreens';
import { TabScreenProps } from '../TabNavigator';

const Stack = createStackNavigator<BrowseStackParamList>();

export default function BrowseStack() {
	return (
		<Stack.Navigator
			initialRouteName="Browse"
			screenOptions={{
				headerStyle: { backgroundColor: tw.color('app-box') },
				headerTintColor: tw.color('ink'),
				headerTitleStyle: tw`text-base`,
				headerBackTitleStyle: tw`text-base`
			}}
		>
			<Stack.Screen name="Browse" component={BrowseScreen} options={{ header: Header }} />
			{SharedScreens(Stack as any)}
		</Stack.Navigator>
	);
}

export type BrowseStackParamList = {
	Browse: undefined;
} & SharedScreensParamList;

export type BrowseStackScreenProps<Screen extends keyof BrowseStackParamList> =
	CompositeScreenProps<
		StackScreenProps<BrowseStackParamList, Screen>,
		TabScreenProps<'BrowseStack'>
	>;
