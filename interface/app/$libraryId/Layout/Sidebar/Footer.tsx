import { Gear } from '@phosphor-icons/react';
import { useNavigate } from 'react-router';
import { useKeys } from 'rooks';
import { JobManagerContextProvider, useClientContext, useDebugState } from '@sd/client';
import { Button, ButtonLink, dialogManager, ModifierKeys, Popover, Tooltip } from '@sd/ui';
import { useOperatingSystem } from '~/hooks';
import { keybindForOs } from '~/util/keybinds';

import DebugPopover from './DebugPopover';
import FeedbackDialog from './FeedbackDialog';
import { IsRunningJob, JobManager } from './JobManager';

export default () => {
	const { library } = useClientContext();
	const debugState = useDebugState();
	const os = useOperatingSystem();
	const keybind = keybindForOs(os);
	const navigate = useNavigate();

	useKeys(['g', 's'], (e) => {
		e.stopPropagation();
		navigate('settings/client/general');
	});

	return (
		<div className="space-y-2">
			<div className="flex w-full items-center justify-between">
				<div className="flex">
					<ButtonLink
						to="settings/client/general"
						size="icon"
						variant="subtle"
						className="text-sidebar-inkFaint ring-offset-sidebar"
					>
						<Tooltip label="Settings" keybinds={'GS'}>
							<Gear className="h-5 w-5" />
						</Tooltip>
					</ButtonLink>
					<JobManagerContextProvider>
						<Popover
							keybind={['Meta', 'j']}
							trigger={
								<Button
									size="icon"
									variant="subtle"
									className="text-sidebar-inkFaint ring-offset-sidebar radix-state-open:bg-sidebar-selected/50"
									disabled={!library}
								>
									{library && (
										<Tooltip
											label="Recent Jobs"
											keybinds={keybind([ModifierKeys.Meta], ['J'])}
										>
											<IsRunningJob />
										</Tooltip>
									)}
								</Button>
							}
						>
							<div className="block h-96 w-[430px]">
								<JobManager />
							</div>
						</Popover>
					</JobManagerContextProvider>
				</div>
				<Button
					variant="outline"
					className="flex items-center gap-1"
					onClick={() => {
						dialogManager.create((dp) => <FeedbackDialog {...dp} />);
					}}
				>
					<p className="text-[11px] font-normal text-sidebar-inkFaint">Feedback</p>
				</Button>
			</div>
			{debugState.enabled && <DebugPopover />}
		</div>
	);
};
