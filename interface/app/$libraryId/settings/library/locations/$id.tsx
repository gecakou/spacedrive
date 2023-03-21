import { useQueryClient } from '@tanstack/react-query';
import { Archive, ArrowsClockwise, Info, Trash } from 'phosphor-react';
import { useParams } from 'react-router';
import { useLibraryMutation, useLibraryQuery } from '@sd/client';
import { Button, Divider, forms, tw } from '@sd/ui';
import { Tooltip } from '@sd/ui';
import ModalLayout from '../../ModalLayout';
import { IndexerRuleEditor } from './IndexerRuleEditor';

const InfoText = tw.p`mt-2 text-xs text-ink-faint`;
const Label = tw.label`mb-1 text-sm font-medium`;
const FlexCol = tw.label`flex flex-col flex-1`;
const ToggleSection = tw.label`flex flex-row w-full`;

const { Form, Input, Switch, useZodForm, z } = forms;

const schema = z.object({
	displayName: z.string(),
	localPath: z.string(),
	indexer_rules_ids: z.array(z.string()),
	generatePreviewMedia: z.boolean(),
	syncPreviewMedia: z.boolean(),
	hidden: z.boolean()
});

export const Component = () => {
	const queryClient = useQueryClient();
	const { id } = useParams<{
		id: string;
	}>();

	useLibraryQuery(['locations.getById', Number(id)], {
		onSuccess: (data) => {
			if (data && !isDirty)
				form.reset({
					displayName: data.name,
					localPath: data.path,
					indexer_rules_ids: data.indexer_rules.map((i) => i.indexer_rule.id.toString()),
					generatePreviewMedia: data.generate_preview_media,
					syncPreviewMedia: data.sync_preview_media,
					hidden: data.hidden
				});
		}
	});

	const form = useZodForm({
		schema
	});

	const updateLocation = useLibraryMutation('locations.update', {
		onError: (e) => console.log({ e }),
		onSuccess: () => {
			form.reset(form.getValues());
			queryClient.invalidateQueries(['locations.list']);
		}
	});

	const onSubmit = form.handleSubmit((data) =>
		updateLocation.mutateAsync({
			id: Number(id),
			name: data.displayName,
			sync_preview_media: data.syncPreviewMedia,
			generate_preview_media: data.generatePreviewMedia,
			hidden: data.hidden,
			indexer_rules_ids: []
		})
	);

	const fullRescan = useLibraryMutation('locations.fullRescan');

	const { isDirty } = form.formState;

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
						<Label>Display Name</Label>
						<Input {...form.register('displayName')} />
						<InfoText>
							The name of this Location, this is what will be displayed in the sidebar. Will not
							rename the actual folder on disk.
						</InfoText>
					</FlexCol>
					<FlexCol>
						<Label>Local Path</Label>
						<Input {...form.register('localPath')} />
						<InfoText>
							The path to this Location, this is where the files will be stored on disk.
						</InfoText>
					</FlexCol>
				</div>
				<Divider />
				<div className="space-y-2">
					<ToggleSection>
						<Label className="grow">Generate preview media for this Location</Label>
						<Switch {...form.register('generatePreviewMedia')} size="sm" />
					</ToggleSection>
					<ToggleSection>
						<Label className="grow">Sync preview media for this Location with your devices</Label>
						<Switch {...form.register('syncPreviewMedia')} size="sm" />
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
				<div className="pointer-events-none flex flex-col opacity-30">
					<Label className="grow">Indexer rules</Label>
					<InfoText className="mt-0 mb-1">
						Indexer rules allow you to specify paths to ignore using RegEx.
					</InfoText>
					<IndexerRuleEditor locationId={id!} />
				</div>
				<Divider />
				<div className="flex space-x-5">
					<FlexCol>
						<div>
							<Button onClick={() => fullRescan.mutate(Number(id))} size="sm" variant="outline">
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
							<Button size="sm" variant="colored" className="border-red-500 bg-red-500 ">
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
