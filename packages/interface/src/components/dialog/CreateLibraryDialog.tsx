import { useBridgeMutation } from '@sd/client';
import { Input } from '@sd/ui';
import { Dialog } from '@sd/ui';
import { useQueryClient } from '@tanstack/react-query';
import { PropsWithChildren } from 'react';
import { useForm } from 'react-hook-form';

export default function CreateLibraryDialog({
	children,
	onSubmit,
	open,
	setOpen
}: PropsWithChildren<{ onSubmit?: () => void; open: boolean; setOpen: (state: boolean) => void }>) {
	const queryClient = useQueryClient();
	const form = useForm({
		defaultValues: {
			name: '',
			password: '' as string,
			secret_key: '' as string | null
		}
	});

	const createLibrary = useBridgeMutation('library.create', {
		onSuccess: (library) => {
			queryClient.setQueryData(['library.list'], (libraries: any) => [
				...(libraries || []),
				library
			]);

			if (onSubmit) onSubmit();
			setOpen(false);
			form.reset();
		},
		onError: (err: any) => {
			console.error(err);
		}
	});
	const doSubmit = form.handleSubmit((data) => {
		if (data.secret_key === '') {
			data.secret_key = null;
		}

		return createLibrary.mutateAsync(data);
	});

	return (
		<Dialog
			open={open}
			setOpen={setOpen}
			title="Create New Library"
			description="Choose a name for your new library, you can configure this and more settings from the library settings later on."
			ctaAction={doSubmit}
			loading={form.formState.isSubmitting}
			submitDisabled={!form.formState.isValid}
			ctaLabel="Create"
			trigger={children}
		>
			<form onSubmit={doSubmit}>
				<div className="relative flex flex-col">
					<p className="text-sm mt-3">Name:</p>
					<Input
						className="flex-grow w-full"
						placeholder="My Cool Library"
						disabled={form.formState.isSubmitting}
						{...form.register('name', { required: true })}
					/>
				</div>

				{/* TODO: Proper UI for this. Maybe checkbox for encrypted or not and then reveal these fields. Select encrypted by default. */}
				{/* <span className="text-sm">Make the secret key field empty to skip key setup.</span> */}

				<div className="relative flex flex-col">
					<p className="text-sm mt-2">Password:</p>
					<Input
						className="flex-grow !py-0.5"
						disabled={form.formState.isSubmitting}
						{...form.register('password')}
						placeholder="Password"
					/>
				</div>
				<div className="relative flex flex-col">
					<p className="text-sm mt-2">Key secret (optional):</p>
					<Input
						className="flex-grow !py-0.5"
						placeholder="0000-0000-0000-0000"
						disabled={form.formState.isSubmitting}
						{...form.register('secret_key')}
					/>
				</div>
			</form>
		</Dialog>
	);
}
