import React from 'react';
import { Text, View } from 'react-native';
import { getDebugState, toggleFeatureFlag } from '@sd/client';
import { Button } from '~/components/primitive/Button';
import { tw } from '~/lib/tailwind';
import { SettingsStackScreenProps } from '~/navigation/SettingsNavigator';

const DebugScreen = ({ navigation }: SettingsStackScreenProps<'Debug'>) => {
	return (
		<View>
			<Text style={tw`text-ink`}>Debug</Text>
			<Button onPress={() => toggleFeatureFlag(['p2pPairing', 'spacedrop'])}>
				<Text style={tw`text-ink`}>Toggle P2P</Text>
			</Button>

			<Button onPress={() => (getDebugState().rspcLogger = !getDebugState().rspcLogger)}>
				<Text style={tw`text-ink`}>Toggle rspc logger</Text>
			</Button>

			<Button
				onPress={() => {
					navigation.popToTop();
					navigation.replace('Home');
					getDebugState().enabled = false;
				}}
			>
				<Text style={tw`text-ink`}>Disable Debug Mode</Text>
			</Button>
		</View>
	);
};

export default DebugScreen;
