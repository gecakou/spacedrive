import { useQueryClient } from '@tanstack/react-query';
import { Archive, ArrowsClockwise, Info, Trash } from 'phosphor-react';
import { Controller } from 'react-hook-form';
import { useParams } from 'react-router';
import { useLibraryMutation, useLibraryQuery } from '@sd/client';
import { Button, Divider, forms, tw } from '@sd/ui';
import { Tooltip } from '@sd/ui';
import { showAlertDialog } from '~/components/AlertDialog';
import ModalLayout from '../../ModalLayout';
import { IndexerRuleEditor } from './IndexerRuleEditor';

const Label = tw.label`mb-1 text-sm font-medium`;
const FlexCol = tw.label`flex flex-col flex-1`;
const InfoText = tw.p`mt-2 text-xs text-ink-faint`;
const ToggleSection = tw.label`flex flex-row w-full`;

const { Form, Input, Switch, useZodForm, z } = forms;

const schema = z.object({
	name: z.string(),
	path: z.string(),
	hidden: z.boolean(),
	indexer_rules_ids: z.array(z.number()),
	sync_preview_media: z.boolean(),
	generate_preview_media: z.boolean()
});

export const Component = () => {
	const form = useZodForm({
		schema,
		defaultValues: {
			indexer_rules_ids: []
		}
	});
	const params = useParams<{ id: string }>();
	const fullRescan = useLibraryMutation('locations.fullRescan');
	const queryClient = useQueryClient();
	const updateLocation = useLibraryMutation('locations.update', {
		onError: (e) => console.log({ e }),
		onSuccess: () => {
			form.reset(form.getValues());
			queryClient.invalidateQueries(['locations.list']);
		}
	});

	const { isDirty } = form.formState;
	const locationId = Number.parseInt(params.id ?? '');
	useLibraryQuery(['locations.getById', locationId], {
		onError: (e) => {
			showAlertDialog({
				title: 'Error',
				value: 'Failed to update location settings'
			});
			console.error({ e });
		},
		onSuccess: (data) => {
			if (data && !isDirty)
				form.reset({
					path: data.path,
					name: data.name,
					hidden: data.hidden,
					indexer_rules_ids: data.indexer_rules.map((i) => i.indexer_rule.id),
					sync_preview_media: data.sync_preview_media,
					generate_preview_media: data.generate_preview_media
				});
		}
	});

	if (Number.isNaN(locationId)) {
		showAlertDialog({
			title: 'Error',
			value: 'Invalid location settings'
		});
		return null;
	}

	const onSubmit = form.handleSubmit(
		({ name, hidden, sync_preview_media, generate_preview_media, indexer_rules_ids }) =>
			updateLocation.mutateAsync({
				id: locationId,
				name,
				hidden,
				indexer_rules_ids,
				sync_preview_media,
				generate_preview_media
			})
	);

	return (
		<Form form={form} onSubmit={onSubmit} className="h-full w-full">
			<ModalLayout
				title="Edit Location"
				topRight={
					<div className="flex flex-row space-x-3">
						{isDirty && (
							<Button onClick={() => form.reset()} variant="outline" size="sm">
								Reset
							</Button>
						)}
						<Button
							type="submit"
							disabled={!isDirty || form.formState.isSubmitting}
							variant={isDirty ? 'accent' : 'outline'}
							size="sm"
						>
							Save Changes
						</Button>
					</div>
				}
			>
				<div className="flex space-x-4">
					<FlexCol>
						<Input label="Display Name" {...form.register('name')} />
						<InfoText>
							The name of this Location, this is what will be displayed in the sidebar. Will not
							rename the actual folder on disk.
						</InfoText>
					</FlexCol>
					<FlexCol>
						<Input
							label="Local Path"
							readOnly={true}
							className="text-ink-dull"
							{...form.register('path')}
						/>
						<InfoText>
							The path to this Location, this is where the files will be stored on disk.
						</InfoText>
					</FlexCol>
				</div>
				<Divider />
				<div className="space-y-2">
					<ToggleSection>
						<Label className="grow">Generate preview media for this Location</Label>
						<Switch {...form.register('generate_preview_media')} size="sm" />
					</ToggleSection>
					<ToggleSection>
						<Label className="grow">Sync preview media for this Location with your devices</Label>
						<Switch {...form.register('sync_preview_media')} size="sm" />
					</ToggleSection>
					<ToggleSection>
						<Label className="grow">
							Hide location and contents from view{' '}
							<Tooltip label='Prevents the location and its contents from appearing in summary categories, search and tags unless "Show hidden items" is enabled.'>
								<Info className="inline" />
							</Tooltip>
						</Label>
						<Switch {...form.register('hidden')} size="sm" />
					</ToggleSection>
				</div>
				<Divider />
				<div className="flex flex-col">
					<Label className="grow">Indexer rules</Label>
					<InfoText className="mt-0 mb-1">
						Indexer rules allow you to specify paths to ignore using RegEx.
					</InfoText>
					<Controller
						name="indexer_rules_ids"
						render={({ field }) => <IndexerRuleEditor field={field} />}
						control={form.control}
					/>
				</div>
				<Divider />
				<div className="flex space-x-5">
					<FlexCol>
						<div>
							<Button onClick={() => fullRescan.mutate(locationId)} size="sm" variant="outline">
								<ArrowsClockwise className="mr-1.5 -mt-0.5 inline h-4 w-4" />
								Full Reindex
							</Button>
						</div>
						<InfoText>Perform a full rescan of this Location.</InfoText>
					</FlexCol>
					<FlexCol>
						<div>
							<Button
								onClick={() => alert('Archiving locations is coming soon...')}
								size="sm"
								variant="outline"
								className=""
							>
								<Archive className="mr-1.5 -mt-0.5 inline h-4 w-4" />
								Archive
							</Button>
						</div>
						<InfoText>
							Extract data from Library as an archive, useful to preserve Location folder structure.
						</InfoText>
					</FlexCol>
					<FlexCol>
						<div>
							<Button size="sm" variant="colored" className="border-red-500 bg-red-500">
								<Trash className="mr-1.5 -mt-0.5 inline h-4 w-4" />
								Delete
							</Button>
						</div>
						<InfoText>
							This will not delete the actual folder on disk. Preview media will be
						</InfoText>
					</FlexCol>
				</div>
				<Divider />
				<div className="h-6" />
			</ModalLayout>
		</Form>
	);
};
