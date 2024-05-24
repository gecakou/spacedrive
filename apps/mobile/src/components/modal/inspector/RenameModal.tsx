import { getIndexedItemFilePath, useLibraryMutation, useRspcLibraryContext } from '@sd/client';
import React, { forwardRef, useEffect, useRef, useState } from 'react';
import { Text, View } from 'react-native';
import { TextInput } from 'react-native-gesture-handler';
import { Modal, ModalRef } from '~/components/layout/Modal';
import { Button } from '~/components/primitive/Button';
import { ModalInput } from '~/components/primitive/Input';
import { toast } from '~/components/primitive/Toast';
import useForwardedRef from '~/hooks/useForwardedRef';
import { tw } from '~/lib/tailwind';
import { useActionsModalStore } from '~/stores/modalStore';

interface Props {
	objectName: string;
}

const RenameModal = forwardRef<ModalRef, Props>((props, ref) => {
	const modalRef = useForwardedRef(ref);
	const [newName, setNewName] = useState('');
	const rspc = useRspcLibraryContext();
	const { data } = useActionsModalStore();
	const inputRef = useRef<TextInput>(null);

	const filePathData = data && getIndexedItemFilePath(data);

	const renameFile = useLibraryMutation(['files.renameFile'], {
		onSuccess: () => {
			modalRef.current?.dismiss();
			rspc.queryClient.invalidateQueries(['search.paths']);
		},
		onError: () => {
			toast.error('Failed to rename object');
		}
	});

	// set input value to object name on initial render
	useEffect(() => {
		setNewName(props.objectName);
	}, [props.objectName]);

	const textRenameHandler = async () => {
			switch (data?.type) {
				case 'Path':
				case 'Object': {
					if (!filePathData) throw new Error('Failed to get file path object');

					const { id, location_id } = filePathData;

					if (!location_id) throw new Error('Missing location id');

					await renameFile.mutateAsync({
						location_id: location_id,
						kind: {
							One: {
								from_file_path_id: id,
								to: newName
							}
						}
					});
					break;
				}
			}
	};

	return (
		<Modal
			ref={modalRef}
			title="Rename"
			onDismiss={() => setNewName(props.objectName)}
			enableContentPanningGesture={false}
			enablePanDownToClose={false}
			snapPoints={['20']}
		>
			<View style={tw`mt-2 flex-col gap-2 px-6`}>
				<ModalInput
					ref={inputRef}
					autoFocus
					onFocus={() => inputRef.current?.setSelection(0, newName.length)}
					value={newName}
					onChangeText={(t) => setNewName(t)}
				/>
				<Button disabled={newName.length === 0 || props.objectName === newName} onPress={textRenameHandler} variant="accent">
					<Text style={tw`font-medium text-ink`}>Save</Text>
				</Button>
			</View>
		</Modal>
	);
});

export default RenameModal;
