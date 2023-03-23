import { ErrorMessage } from '@hookform/error-message';
import { RSPCError } from '@rspc/client';
import { ChangeEvent, useEffect, useState } from 'react';
import { Controller, UseFormReturn } from 'react-hook-form';
import { useLibraryMutation, useLibraryQuery } from '@sd/client';
import { CheckBox, Dialog, UseDialogProps, useDialog } from '@sd/ui';
import { Input, useZodForm, z } from '@sd/ui/src/forms';
import { showAlertDialog } from '~/components/AlertDialog';
import { Platform, usePlatform } from '~/util/Platform';

const schema = z.object({ path: z.string(), indexer_rules_ids: z.array(z.number()) });

type FormFieldValues<U> = U extends UseFormReturn<infer U> ? U : never;

interface Props extends UseDialogProps {
	path: string;
}

const REMOTE_ERROR_FORM_FIELD = 'root.serverError';
const REMOTE_ERROR_FORM_MESSAGES = {
	// \u000A is a line break, It works with css white-space: pre-line
	ADD_LIBRARY:
		'Location is already linked to another Library.\u000ADo you want to add it to this Library too?',
	NEED_RELINK: 'Location already present.\u000ADo you want to relink it?'
};

type RemoteErrorFormMessage = keyof typeof REMOTE_ERROR_FORM_MESSAGES;

const isRemoteErrorFormMessage = (message: unknown): message is RemoteErrorFormMessage =>
	typeof message === 'string' && Object.hasOwnProperty.call(REMOTE_ERROR_FORM_MESSAGES, message);

export const openDirectoryPickerDialog = async (platform: Platform): Promise<null | string> => {
	if (!platform.openDirectoryPickerDialog) return null;

	const path = await platform.openDirectoryPickerDialog();
	if (!path) return '';
	if (typeof path !== 'string')
		// TODO: Should adding multiple locations simultaneously be implemented?
		throw new Error('Adding multiple locations simultaneously is not supported');

	return path;
};

export const AddLocationDialog = (props: Props) => {
	const dialog = useDialog(props);
	const platform = usePlatform();
	const createLocation = useLibraryMutation('locations.create');
	const relinkLocation = useLibraryMutation('locations.relink');
	const indexerRulesList = useLibraryQuery(['locations.indexer_rules.list']);
	const addLocationToLibrary = useLibraryMutation('locations.addLibrary');
	const [remoteError, setRemoteError] = useState<null | RemoteErrorFormMessage>(null);

	const form = useZodForm({
		schema,
		defaultValues: {
			path: props.path,
			indexer_rules_ids: []
		}
	});

	// Block to prevent single-use constants poluting parent scope
	{
		// Destructuring so eslint stop complaining about useEffect missing `form` dependency
		const { watch, clearErrors } = form;
		useEffect(() => {
			// Clear custom remote error when user performs any change on the form
			const subscription = watch(() => {
				clearErrors(REMOTE_ERROR_FORM_FIELD);
				setRemoteError(null);
			});
			return () => subscription.unsubscribe();
		}, [watch, clearErrors]);
	}

	const onLocationSubmit = async ({ path, indexer_rules_ids }: FormFieldValues<typeof form>) => {
		switch (remoteError) {
			case null:
				await createLocation.mutateAsync({ path, indexer_rules_ids });
				break;
			case 'NEED_RELINK':
				await relinkLocation.mutateAsync(path);
				// TODO: Update relinked location with new indexer rules, don't have a way to get location id yet though
				// await updateLocation.mutateAsync({
				// 	id: locationId,
				// 	name: null,
				// 	hidden: null,
				// 	indexer_rules_ids,
				// 	sync_preview_media: null,
				// 	generate_preview_media: null
				// });
				break;
			case 'ADD_LIBRARY':
				await addLocationToLibrary.mutateAsync({ path, indexer_rules_ids });
				break;
			default:
				throw new Error('Unimplemented custom remote error handling');
		}
	};

	const onLocationSubmitError = async (error: Error) => {
		if ('cause' in error && error.cause instanceof RSPCError) {
			// TODO: error.code property is not yet implemented in RSPCError
			// https://github.com/oscartbeaumont/rspc/blob/60a4fa93187c20bc5cb565cc6ee30b2f0903840e/packages/client/src/interop/error.ts#L59
			// So we grab it from the shape for now
			const { code } = error.cause.shape;
			if (code !== 500) {
				let { message } = error;

				if (code == 409 && isRemoteErrorFormMessage(message)) {
					setRemoteError(message);
					message = REMOTE_ERROR_FORM_MESSAGES[message];

					/**
					 * TODO: On NEED_RELINK, we should query the backend for
					 * the current location indexer_rules_ids, then update the checkboxes
					 * accordingly. However we don't have the location id at this point.
					 * Maybe backend could return the location id in the error?
					 */
				}

				form.reset({}, { keepValues: true, keepErrors: true, keepIsValid: true });
				form.setError(REMOTE_ERROR_FORM_FIELD, { type: 'remote', message: message });

				// Throw error to prevent dialog from closing
				throw error;
			}
		}

		showAlertDialog({
			title: 'Error',
			value: error.message || 'Failed to add location'
		});
	};

	return (
		<Dialog
			{...{ dialog, form }}
			title="New Location"
			description={
				platform.platform === 'web'
					? 'As you are using the browser version of Spacedrive you will (for now) need to specify an absolute URL of a directory local to the remote node.'
					: ''
			}
			onSubmit={form.handleSubmit((fields) =>
				onLocationSubmit(fields).catch(onLocationSubmitError)
			)}
			ctaLabel="Add"
		>
			<div className="relative mb-3 flex flex-col">
				<p className="my-2 text-sm font-bold">Path:</p>
				<Input
					type="text"
					onClick={() =>
						openDirectoryPickerDialog(platform)
							.then((path) => path && form.setValue('path', path))
							.catch((error) => showAlertDialog({ title: 'Error', value: String(error) }))
					}
					readOnly={platform.platform !== 'web'}
					required
					className="grow cursor-pointer !py-0.5"
					{...form.register('path')}
				/>
			</div>

			<div className="relative mb-3 flex flex-col">
				<p className="my-2 text-sm font-bold">File indexing rules:</p>
				<div className="mb-3 grid w-full grid-cols-2 gap-4">
					<Controller
						name="indexer_rules_ids"
						control={form.control}
						render={({ field }) => (
							<>
								{indexerRulesList.data?.map((rule) => (
									<div className="flex" key={rule.id}>
										<CheckBox
											value={rule.id}
											checked={field.value.includes(rule.id)}
											onChange={(event: ChangeEvent) => {
												const checkBoxRef = event.target as HTMLInputElement;
												const checkBoxValue = Number.parseInt(checkBoxRef.value);
												if (checkBoxRef.checked) {
													field.onChange([...field.value, checkBoxValue]);
												} else {
													field.onChange(
														field.value.filter((fieldValue) => fieldValue !== checkBoxValue)
													);
												}
											}}
											className="bg-app-selected"
										/>
										<span className="mt-1 text-xs font-medium">{rule.name}</span>
									</div>
								))}
							</>
						)}
					/>
				</div>
			</div>

			<ErrorMessage
				name={REMOTE_ERROR_FORM_FIELD}
				render={({ message }) => (
					<span className="inline-block w-full whitespace-pre-wrap text-center text-sm font-semibold text-red-500">
						{message}
					</span>
				)}
			/>
		</Dialog>
	);
};
