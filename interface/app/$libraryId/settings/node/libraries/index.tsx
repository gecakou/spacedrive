import { useRef } from 'react';
import { useNavigate } from 'react-router';
import { useBridgeQuery, useClientContext, useFeatureFlag, useLibraryContext, useZodForm } from '@sd/client';
import { Button, Dialog, dialogManager, useDialog, z } from '@sd/ui';
import { useLocale } from '~/hooks';

import { Heading } from '../../Layout';
import CreateDialog from './CreateDialog';
import JoinDialog from './JoinDialog';
import ListItem from './ListItem';

export const Component = () => {
	const librariesQuery = useBridgeQuery(['library.list']);
	const cloudLibrariesQuery = useBridgeQuery(['cloud.library.list']);
	const libraries = librariesQuery.data;
	const cloudLibraries = cloudLibrariesQuery.data;

	const cloudEnabled = useFeatureFlag('cloudSync');

	const { library } = useLibraryContext();
	const { libraries: librariesCtx } = useClientContext();
	const librariesCtxData = librariesCtx.data;

	const { t } = useLocale();

	return (
		<>
			<Heading
				title={t('libraries')}
				description={t('libraries_description')}
				rightArea={
					<div className="flex-row space-x-2">
						<Button
							variant="accent"
							size="sm"
							onClick={() => {
								dialogManager.create((dp) => <CreateDialog {...dp} />);
							}}
						>
							{t('add_library')}
						</Button>
						{cloudEnabled && (
							<Button
								variant="accent"
								size="sm"
								onClick={() => {
									dialogManager.create((dp) => <JoinDialog librariesCtx={librariesCtxData} {...dp} />);
								}}
							>
								{t('join_library')}
							</Button>
						)}
					</div>
				}
			/>
			<div className="space-y-2">
				{libraries
					?.sort((a, b) => {
						if (a.uuid === library.uuid) return -1;
						if (b.uuid === library.uuid) return 1;
						return 0;
					})
					.map((lib) => (
						<ListItem
							current={lib.uuid === library.uuid}
							key={lib.uuid}
							library={lib}
						/>
					))}
			</div>
			<div className="space-y-2">
				<Heading title={'Cloud Libraries'} description={'Cloud Libraries'} />
				{cloudLibraries
					?.sort((a, b) => {
						if (a.uuid === library.uuid) return -1;
						if (b.uuid === library.uuid) return 1;
						return 0;
					})
					.map((lib) => (
						<ListItem
							current={lib.uuid === library.uuid}
							key={lib.uuid}
							library={library}
						/>
					))}
			</div>
		</>
	);
};
