import { useDrawerProgress } from '@react-navigation/drawer';
import React from 'react';
import Animated, { Extrapolate, interpolate, useAnimatedStyle } from 'react-native-reanimated';

import tw from '../../lib/tailwind';

const DrawerScreenWrapper: React.FC = ({ children }) => {
	const progress: any = useDrawerProgress();

	const style = useAnimatedStyle(() => {
		const scale = interpolate(progress.value, [0, 1], [1, 0.85], Extrapolate.CLAMP);
		const translateX = interpolate(progress.value, [0, 1], [0, -20], Extrapolate.CLAMP);
		const borderRadius = interpolate(progress.value, [0, 1], [0, 16], Extrapolate.CLAMP);
		return {
			transform: [{ scale: scale }, { translateX }],
			borderRadius
		};
	}, []);
	return (
		<Animated.View style={[tw.style('flex-1 items-center justify-center bg-[#121219]'), style]}>
			{children}
		</Animated.View>
	);
};

export default DrawerScreenWrapper;
