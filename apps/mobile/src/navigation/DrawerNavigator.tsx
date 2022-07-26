import { createDrawerNavigator } from '@react-navigation/drawer';

import DrawerContent from '../components/drawer/DrawerContent';
import LocationScreen from '../screens/Location';
import OverviewScreen from '../screens/Overview';
import PhotosScreen from '../screens/Photos';
import SpacesScreen from '../screens/Spaces';
import TagScreen from '../screens/Tag';
import SettingsScreen from '../screens/settings/Settings';
import { HomeDrawerParamList } from '../types/navigation';

const Drawer = createDrawerNavigator<HomeDrawerParamList>();

// TODO: Implement Animated Drawer (maybe scale down + blur the screen when drawer is open)
// TODO: Implement Animated Height to expand Locations & Tags
// TODO: Custom Header with Search and Button to open drawer

export default function DrawerNavigator() {
	return (
		<Drawer.Navigator
			initialRouteName="Overview"
			screenOptions={{
				headerShown: false,
				drawerStyle: {
					backgroundColor: '#08090D',
					width: '75%'
				}
				// drawerHideStatusBarOnOpen: true,
				// drawerStatusBarAnimation: 'slide'
			}}
			drawerContent={(props) => <DrawerContent {...(props as any)} />}
		>
			<Drawer.Screen name="Overview" component={OverviewScreen} />
			<Drawer.Screen name="Spaces" component={SpacesScreen} />
			<Drawer.Screen name="Photos" component={PhotosScreen} />
			<Drawer.Screen name="Location" component={LocationScreen} />
			<Drawer.Screen name="Tag" component={TagScreen} />
			<Drawer.Screen name="Settings" component={SettingsScreen} />
		</Drawer.Navigator>
	);
}
