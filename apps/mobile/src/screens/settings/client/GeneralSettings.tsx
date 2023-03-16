import { Text, View } from 'react-native';
import { useBridgeQuery } from '@sd/client';
import { Input } from '~/components/form/Input';
import Card from '~/components/layout/Card';
import { Divider } from '~/components/primitive/Divider';
import { SettingsInputTitle } from '~/components/settings/SettingsContainer';
import { tw } from '~/lib/tailwind';
import { SettingsStackScreenProps } from '~/navigation/SettingsNavigator';

const GeneralSettingsScreen = ({ navigation }: SettingsStackScreenProps<'GeneralSettings'>) => {
	const { data: node } = useBridgeQuery(['nodeState']);

	if (!node) return null;

	return (
		<View style={tw`flex-1 p-4`}>
			<Card style={tw`bg-app-box`}>
				{/* Card Header */}
				<View style={tw`flex flex-row justify-between`}>
					<Text style={tw`text-ink font-semibold`}>Connected Node</Text>
					<View style={tw`flex flex-row`}>
						{/* Peers */}
						<View style={tw`bg-app-highlight mr-2 self-start rounded px-1.5 py-[2px]`}>
							<Text style={tw`text-ink text-xs font-semibold`}>0 Peers</Text>
						</View>
						{/* Status */}
						<View style={tw`bg-accent rounded px-1.5 py-[2px]`}>
							<Text style={tw`text-ink text-xs font-semibold`}>Running</Text>
						</View>
					</View>
				</View>
				{/* Divider */}
				<Divider style={tw`mt-2 mb-4`} />
				{/* Node Name and Port */}
				<SettingsInputTitle>Node Name</SettingsInputTitle>
				<Input value={node.name} />
				<SettingsInputTitle style={tw`mt-3`}>Node Port</SettingsInputTitle>
				<Input value={node.p2p_port?.toString() ?? '5795'} keyboardType="numeric" />
			</Card>
		</View>
	);
};

export default GeneralSettingsScreen;
