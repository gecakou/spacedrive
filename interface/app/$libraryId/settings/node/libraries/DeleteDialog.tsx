import { useQueryClient } from '@tanstack/react-query';
import { useBridgeMutation, usePlausibleEvent } from '@sd/client';
import { Dialog, UseDialogProps, forms, useDialog } from '@sd/ui';

const { useZodForm, z } = forms;

interface Props extends UseDialogProps {
	libraryUuid: string;
}

export default function DeleteLibraryDialog(props: Props) {
	const submitPlausibleEvent = usePlausibleEvent();
	const queryClient = useQueryClient();
	const deleteLib = useBridgeMutation('library.delete', {
		onSuccess: () => {
			queryClient.invalidateQueries(['library.list']);

			submitPlausibleEvent({
				event: {
					type: 'libraryDelete'
				}
			});
		},
		onError: (e) => {
			alert(`Failed to delete library: ${e}`);
		}
	});

	const form = useZodForm({ schema: z.object({}) });

	return (
		<Dialog
			form={form}
			onSubmit={form.handleSubmit(() => deleteLib.mutateAsync(props.libraryUuid))}
			dialog={useDialog(props)}
			title="Delete Library"
			description="Deleting a library will permanently the database, the files themselves will not be deleted."
			ctaDanger
			ctaLabel="Delete"
		/>
	);
}
