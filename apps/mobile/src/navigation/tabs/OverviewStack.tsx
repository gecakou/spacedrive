import { CompositeScreenProps } from '@react-navigation/native';
import {
	HeaderStyleInterpolators,
	StackScreenProps,
	TransitionPresets,
	createStackNavigator
} from '@react-navigation/stack';

import OverviewScreen from '../../screens/Overview';
import { SharedScreens, SharedScreensParamList } from '../SharedScreens';
import { TabScreenProps } from '../TabNavigator';

const Stack = createStackNavigator<OverviewStackParamList>();

export default function OverviewStack() {
	return (
		<Stack.Navigator
			initialRouteName="Overview"
			screenOptions={{
				headerStyle: { backgroundColor: '#08090D' },
				headerTintColor: '#fff',
				headerStyleInterpolator: HeaderStyleInterpolators.forUIKit,
				...TransitionPresets.DefaultTransition
			}}
		>
			<Stack.Screen name="Overview" component={OverviewScreen} />
			{SharedScreens(Stack as any)}
		</Stack.Navigator>
	);
}

export type OverviewStackParamList = {
	Overview: undefined;
} & SharedScreensParamList;

export type OverviewStackScreenProps<Screen extends keyof OverviewStackParamList> =
	CompositeScreenProps<
		StackScreenProps<OverviewStackParamList, Screen>,
		TabScreenProps<'OverviewStack'>
	>;
