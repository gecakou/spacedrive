import {
	DrawerNavigationState,
	getFocusedRouteNameFromRoute,
	ParamListBase,
	Route
} from '@react-navigation/native';
import { valtioPersist } from '@sd/client';

export const currentLibraryStore = valtioPersist('sdActiveLibrary', {
	id: null as string | null
});

export const getActiveRouteFromState = (state: any): Partial<Route<string, object | undefined>> => {
	if (!state.routes || state.routes.length === 0 || state.index >= state.routes.length) {
		return state;
	}
	const childActiveRoute = state.routes[state.index];
	return getActiveRouteFromState(childActiveRoute);
};

export const getStackNameFromState = (state: DrawerNavigationState<ParamListBase>) => {
	return getFocusedRouteNameFromRoute(getActiveRouteFromState(state)) ?? 'OverviewStack';
};
