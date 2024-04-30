import { useNavigation } from '@react-navigation/native';
import { DotsThreeOutline, Plus } from 'phosphor-react-native';
import { useRef } from 'react';
import { Text, View } from 'react-native';
import { useLibraryQuery } from '@sd/client';
import { ModalRef } from '~/components/layout/Modal';
import { tw } from '~/lib/tailwind';
import { BrowseStackScreenProps } from '~/navigation/tabs/BrowseStack';
import { SettingsStackScreenProps } from '~/navigation/tabs/SettingsStack';

import Empty from '../layout/Empty';
import { LocationItem } from '../locations/LocationItem';
import ImportModal from '../modal/ImportModal';
import { Button } from '../primitive/Button';

const BrowseLocations = () => {
	const navigation = useNavigation<
		BrowseStackScreenProps<'Browse'>['navigation'] &
			SettingsStackScreenProps<'Settings'>['navigation']
	>();

	const modalRef = useRef<ModalRef>(null);

	const result = useLibraryQuery(['locations.list'], { keepPreviousData: true });
	const locations = result.data;

	return (
		<View style={tw`gap-5 px-6`}>
			<View style={tw`w-full flex-row items-center justify-between`}>
				<Text style={tw`text-lg font-bold text-white`}>Locations</Text>
				<View style={tw`flex-row gap-3`}>
					<Button
						style={tw`h-9 w-9 rounded-full`}
						variant="dashed"
						onPress={() => modalRef.current?.present()}
					>
						<Plus weight="bold" size={16} style={tw`text-ink`} />
					</Button>
					<Button
						onPress={() => {
							navigation.navigate('Locations');
						}}
						style={tw`h-9 w-9 rounded-full`}
						variant="gray"
					>
						<DotsThreeOutline weight="fill" size={16} color={'white'} />
					</Button>
				</View>
			</View>
			<View style={tw`flex-row flex-wrap gap-2`}>
				{locations?.length === 0 ? (
					<Empty description="You have not added any locations" icon="Folder" />
				) : (
					<>
						{locations?.slice(0, 3).map((location) => (
							<LocationItem
								key={location.id}
								location={location}
								editLocation={() =>
									navigation.navigate('SettingsStack', {
										screen: 'EditLocationSettings',
										params: { id: location.id },
										initial: false
									})
								}
								onPress={() => navigation.navigate('Location', { id: location.id })}
							/>
						))}
					</>
				)}
			</View>
			<ImportModal ref={modalRef} />
		</View>
	);
};

export default BrowseLocations;
