import { useQueryClient } from '@tanstack/react-query';
import { useState } from 'react';
import { useBridgeMutation, usePlausibleEvent } from '@sd/client';
import { Input } from '~/components/form/Input';
import Dialog from '~/components/layout/Dialog';
import { currentLibraryStore } from '~/utils/nav';

type Props = {
	onSubmit?: () => void;
	disableBackdropClose?: boolean;
	children: React.ReactNode;
};

// TODO: Move to a Modal component
const CreateLibraryDialog = ({ children, onSubmit, disableBackdropClose }: Props) => {
	const queryClient = useQueryClient();
	const [libName, setLibName] = useState('');
	const [isOpen, setIsOpen] = useState(false);

	const submitPlausibleEvent = usePlausibleEvent();

	const { mutate: createLibrary, isLoading: createLibLoading } = useBridgeMutation(
		'library.create',
		{
			onSuccess: (lib) => {
				// Reset form
				setLibName('');

				// We do this instead of invalidating the query because it triggers a full app re-render??
				queryClient.setQueryData(['library.list'], (libraries: any) => [...(libraries || []), lib]);

				// Switch to the new library
				currentLibraryStore.id = lib.uuid;

				submitPlausibleEvent({ event: { type: 'libraryCreate' } });

				onSubmit?.();
			},
			onSettled: () => {
				// Close create lib dialog
				setIsOpen(false);
			}
		}
	);
	return (
		<Dialog
			isVisible={isOpen}
			setIsVisible={setIsOpen}
			title="Create New Library"
			description="Choose a name for your new library, you can configure this and more settings from the library settings later on."
			ctaLabel="Create"
			ctaAction={() =>
				createLibrary({
					name: libName,
					// TODO: Support password and secret on mobile
					auth: {
						type: 'Password',
						value: ''
					},
					algorithm: 'XChaCha20Poly1305',
					hashing_algorithm: { name: 'Argon2id', params: 'Standard' }
				})
			}
			loading={createLibLoading}
			ctaDisabled={libName.length === 0}
			trigger={children}
			disableBackdropClose={disableBackdropClose}
			onClose={() => setLibName('')} // Resets form onClose
		>
			<Input
				value={libName}
				onChangeText={(text) => setLibName(text)}
				placeholder="My Cool Library"
			/>
		</Dialog>
	);
};

export default CreateLibraryDialog;
