import { useNavigation } from '@react-navigation/native';
import { useQueryClient } from '@tanstack/react-query';
import { forwardRef, useState } from 'react';
import { Text, View } from 'react-native';
import {
	insertLibrary,
	useBridgeMutation,
	useBridgeQuery,
	useClientContext,
	usePlausibleEvent
} from '@sd/client';
import { Modal, ModalRef } from '~/components/layout/Modal';
import { Button } from '~/components/primitive/Button';
import { ModalInput } from '~/components/primitive/Input';
import useForwardedRef from '~/hooks/useForwardedRef';
import { tw } from '~/lib/tailwind';
import { currentLibraryStore } from '~/utils/nav';

const ImportModalLibrary = forwardRef<ModalRef, unknown>((_, ref) => {
	const navigation = useNavigation();
	const modalRef = useForwardedRef(ref);

	const queryClient = useQueryClient();
	const { libraries } = useClientContext();
	const [libName, setLibName] = useState('');

	const submitPlausibleEvent = usePlausibleEvent();

	const cloudLibraries = useBridgeQuery(['cloud.library.list']);
	const joinLibrary = useBridgeMutation(['cloud.library.join']);

	// const { mutate: createLibrary, isLoading: createLibLoading } = useBridgeMutation(
	// 	'library.create',
	// 	{
	// 		onSuccess: (lib) => {
	// 			// Reset form
	// 			setLibName('');

	// 			// We do this instead of invalidating the query because it triggers a full app re-render??
	// 			insertLibrary(queryClient, lib);

	// 			// Switch to the new library
	// 			currentLibraryStore.id = lib.uuid;

	// 			submitPlausibleEvent({ event: { type: 'libraryCreate' } });
	// 		},
	// 		onSettled: () => {
	// 			modalRef.current?.dismiss();
	// 		}
	// 	}
	// );

	if (cloudLibraries.isLoading)
		return (
			<Modal
				ref={modalRef}
				snapPoints={['30']}
				title="Join a Cloud Library"
				description="Connect to one of your cloud libraries."
				onDismiss={() => {
					// Resets form onDismiss
					setLibName('');
				}}
				showCloseButton
				// Disable panning gestures
				enableHandlePanningGesture={false}
				enableContentPanningGesture={false}
			>
				<View style={tw`px-4`}>
					<Text>Loading...</Text>
					<Button
						variant="accent"
						onPress={() => console.log('TODO')}
						style={tw`mt-4`}
						disabled
					>
						<Text style={tw`text-sm font-medium text-white`}>Import</Text>
					</Button>
				</View>
			</Modal>
		);

	return (
		<Modal
			ref={modalRef}
			snapPoints={['30']}
			title="Join a Cloud Library"
			description="Connect to one of your cloud libraries."
			onDismiss={() => {
				// Resets form onDismiss
				setLibName('');
			}}
			showCloseButton
			// Disable panning gestures
			enableHandlePanningGesture={false}
			enableContentPanningGesture={false}
		>
			<View style={tw`gap-y-2 px-4`}>
				{cloudLibraries.data
					?.filter(
						(cloudLibrary) => !libraries.data?.find((l) => l.uuid === cloudLibrary.uuid)
					)
					.map((cloudLibrary) => (
						<View
							key={cloudLibrary.uuid}
							style={tw`flex flex-row items-center gap-2 rounded-lg bg-gray-600 p-2`}
						>
							<Text style={tw`text-xl font-bold text-ink`}>{cloudLibrary.name}</Text>
							<View style={tw`flex flex-row gap-2`}>
								<Button
									variant="accent"
									disabled={joinLibrary.isLoading}
									onPress={async () => {
										const library = await joinLibrary.mutateAsync(
											cloudLibrary.uuid
										);

										queryClient.setQueryData(
											['library.list'],
											(libraries: any) => {
												// The invalidation system beat us to it
												if (
													(libraries || []).find(
														(l: any) => l.uuid === library.uuid
													)
												)
													return libraries;

												return [...(libraries || []), library];
											}
										);

										currentLibraryStore.id = library.uuid;

										navigation.navigate('Root', {
											screen: 'Home',
											params: {
												screen: 'OverviewStack',
												params: {
													screen: 'Overview'
												}
											}
										});

										modalRef.current?.dismiss();
									}}
								>
									<Text style={tw`text-sm font-medium text-white`}>
										{joinLibrary.isLoading &&
										joinLibrary.variables === cloudLibrary.uuid
											? 'Joining...'
											: 'Join'}
									</Text>
								</Button>
							</View>
						</View>
					))}
			</View>
		</Modal>
	);
});

export default ImportModalLibrary;
