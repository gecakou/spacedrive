import { useEffect, useMemo } from 'react';
import {
	CRDTOperation,
	CRDTOperationData,
	useLibraryMutation,
	useLibraryQuery,
	useLibrarySubscription,
	useZodForm
} from '@sd/client';
import { Button, Dialog, dialogManager, useDialog, UseDialogProps, z } from '@sd/ui';
import { useRouteTitle } from '~/hooks/useRouteTitle';

type MessageGroup = {
	model: number;
	id: string;
	messages: { data: CRDTOperationData; timestamp: number }[];
};

export const Component = () => {
	useRouteTitle('Sync');

	const syncEnabled = useLibraryQuery(['sync.enabled']);

	const messages = useLibraryQuery(['sync.messages']);
	const backfillSync = useLibraryMutation(['sync.backfill'], {
		onSuccess: async () => {
			await syncEnabled.refetch();
			await messages.refetch();
		}
	});

	useLibrarySubscription(['sync.newMessage'], {
		onData: () => messages.refetch()
	});

	const groups = useMemo(
		() => (messages.data && calculateGroups(messages.data)) || [],
		[messages]
	);

	return (
		<ul className="space-y-4 p-4">
			{!syncEnabled.data && (
				<Button
					variant="accent"
					onClick={() => {
						dialogManager.create((dialogProps) => (
							<SyncBackfillDialog {...dialogProps} />
						));
					}}
					disabled={backfillSync.isLoading}
				>
					Enable sync messages
				</Button>
			)}
			{groups?.map((group, index) => <OperationGroup key={index} group={group} />)}
		</ul>
	);
};

const OperationGroup = ({ group }: { group: MessageGroup }) => {
	const [header, contents] = (() => {
		const header = (
			<div className="flex items-center space-x-2 p-2">
				<span>{group.model}</span>
				<span className="">{group.id}</span>
			</div>
		);
		const contents = (
			<ul className="flex flex-col space-y-2 p-2">
				{group.messages.map((message, index) => (
					<li key={index} className="flex flex-row justify-between px-2">
						{typeof message.data === 'string' ? (
							<p>Delete</p>
						) : 'u' in message.data ? (
							<p>Update - {message.data.u.field}</p>
						) : (
							<div>
								<p>Create</p>
								<ul>
									{Object.entries(message.data.c).map(([key, value]) => (
										<li className="pl-2" key={key}>
											{key}: {JSON.stringify(value)}
										</li>
									))}
								</ul>
							</div>
						)}
						<p className="text-gray-400">{message.timestamp}</p>
					</li>
				))}
			</ul>
		);
		return [header, contents];
	})();

	return (
		<div className="divide-y divide-gray bg-app-darkBox">
			{header}
			{contents}
		</div>
	);
};

function calculateGroups(messages: CRDTOperation[]) {
	return messages.reduce<MessageGroup[]>((acc, op) => {
		const { data } = op;

		const id = JSON.stringify(op.record_id);

		const latest = (() => {
			const latest = acc[acc.length - 1];

			if (!latest || latest.model !== op.model || latest.id !== id) {
				const group: MessageGroup = {
					model: op.model,
					id,
					messages: []
				};

				acc.push(group);

				return group;
			} else return latest;
		})();

		latest.messages.push({
			data,
			timestamp: op.timestamp
		});

		return acc;
	}, []);
}

function SyncBackfillDialog(props: UseDialogProps) {
	const form = useZodForm({ schema: z.object({}) });
	const dialog = useDialog(props);

	const enableSync = useLibraryMutation(['sync.backfill'], {});

	// dialog is in charge of enabling sync
	useEffect(() => {
		form.handleSubmit(
			async () => {
				await enableSync.mutateAsync(null).then(() => (dialog.state.open = false));
			},
			() => {}
		)();
		// eslint-disable-next-line react-hooks/exhaustive-deps
	}, []);

	return (
		<Dialog
			title="Backfilling Sync Operations"
			description="Library is paused until backfill completes"
			form={form}
			dialog={dialog}
			hideButtons
			ignoreClickOutside
		/>
	);
}
