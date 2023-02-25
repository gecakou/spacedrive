import { Database, DotsSixVertical, Pencil, Trash } from 'phosphor-react';
import { useBridgeQuery, useLibraryContext } from '@sd/client';
import { LibraryConfigWrapped } from '@sd/client';
import { Button, ButtonLink, Card, dialogManager, tw } from '@sd/ui';
import CreateLibraryDialog from '~/components/dialog/CreateLibraryDialog';
import DeleteLibraryDialog from '~/components/dialog/DeleteLibraryDialog';
import { SettingsContainer } from '~/components/settings/SettingsContainer';
import { SettingsHeader } from '~/components/settings/SettingsHeader';
import { Tooltip } from '~/components/tooltip/Tooltip';

const Pill = tw.span`px-1.5 ml-2 py-[2px] rounded text-xs font-medium bg-accent`;

function LibraryListItem(props: { library: LibraryConfigWrapped; current: boolean }) {
	return (
		<Card>
			<DotsSixVertical weight="bold" className="mt-[15px] mr-3 opacity-30" />
			<div className="my-0.5 flex-1">
				<h3 className="font-semibold">
					{props.library.config.name}
					{props.current && <Pill>Current</Pill>}
				</h3>
				<p className="text-ink-dull mt-0.5 text-xs">{props.library.uuid}</p>
			</div>
			<div className="flex flex-row items-center space-x-2">
				<Button className="!p-1.5" variant="gray">
					<Tooltip label="TODO">
						<Database className="h-4 w-4" />
					</Tooltip>
				</Button>
				<ButtonLink className="!p-1.5" to="../library" variant="gray">
					<Tooltip label="Edit Library">
						<Pencil className="h-4 w-4" />
					</Tooltip>
				</ButtonLink>
				<Button
					className="!p-1.5"
					variant="gray"
					onClick={() => {
						dialogManager.create((dp) => (
							<DeleteLibraryDialog {...dp} libraryUuid={props.library.uuid} />
						));
					}}
				>
					<Tooltip label="Delete Library">
						<Trash className="h-4 w-4" />
					</Tooltip>
				</Button>
			</div>
		</Card>
	);
}

export default function LibrarySettings() {
	const libraries = useBridgeQuery(['library.list']);

	const { library } = useLibraryContext();

	return (
		<SettingsContainer>
			<SettingsHeader
				title="Libraries"
				description="The database contains all library data and file metadata."
				rightArea={
					<div className="flex-row space-x-2">
						<Button
							variant="accent"
							size="sm"
							onClick={() => {
								dialogManager.create((dp) => <CreateLibraryDialog {...dp} />);
							}}
						>
							Add Library
						</Button>
					</div>
				}
			/>

			<div className="space-y-2">
				{libraries.data
					?.sort((a, b) => {
						if (a.uuid === library.uuid) return -1;
						if (b.uuid === library.uuid) return 1;
						return 0;
					})
					.map((library) => (
						<LibraryListItem
							current={library.uuid === library.uuid}
							key={library.uuid}
							library={library}
						/>
					))}
			</div>
		</SettingsContainer>
	);
}
