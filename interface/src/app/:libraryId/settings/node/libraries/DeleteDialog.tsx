import { useQueryClient } from '@tanstack/react-query';
import { useBridgeMutation } from '@sd/client';
import { Dialog, UseDialogProps, useDialog } from '@sd/ui';
import { forms } from '@sd/ui';

const { useZodForm, z } = forms;

interface Props extends UseDialogProps {
	libraryUuid: string;
}

export default function DeleteLibraryDialog(props: Props) {
	const dialog = useDialog(props);

	const queryClient = useQueryClient();
	const deleteLib = useBridgeMutation('library.delete', {
		onSuccess: () => {
			queryClient.invalidateQueries(['library.list']);
		}
	});

	const form = useZodForm({ schema: z.object({}) });

	const onSubmit = form.handleSubmit(() => deleteLib.mutateAsync(props.libraryUuid));

	return (
		<Dialog
			form={form}
			onSubmit={onSubmit}
			dialog={dialog}
			title="Delete Library"
			description="Deleting a library will permanently the database, the files themselves will not be deleted."
			ctaDanger
			ctaLabel="Delete"
		/>
	);
}
