import { CheckSquare } from '@phosphor-icons/react';
import { useQueryClient } from '@tanstack/react-query';
import { useNavigate } from 'react-router';
import {
	auth,
	backendFeatures,
	features,
	toggleFeatureFlag,
	useBridgeQuery,
	useDebugState,
	useFeatureFlags,
	useLibraryMutation
} from '@sd/client';
import { useBridgeMutation } from "@sd/client"
import {
	Button,
	Dropdown,
	DropdownMenu,
	Popover,
	Select,
	SelectOption,
	Switch,
	usePopover
} from '@sd/ui';
import { toggleRenderRects } from '~/hooks';
import { usePlatform } from '~/util/Platform';

import Setting from '../../settings/Setting';

export default () => {
	const buildInfo = useBridgeQuery(['buildInfo']);
	const nodeState = useBridgeQuery(['nodeState']);

	const debugState = useDebugState();
	const platform = usePlatform();
	const navigate = useNavigate();

	return (
		<Popover
			popover={usePopover()}
			className="p-4 focus:outline-none"
			trigger={
				<h1 className="ml-1 w-full text-[7pt] text-sidebar-inkFaint/50">
					v{buildInfo.data?.version || '-.-.-'} - {buildInfo.data?.commit || 'dev'}
				</h1>
			}
		>
			<div className="no-scrollbar block h-96 w-[430px] overflow-y-scroll pb-4">
				<Setting mini title="Cloud Origin" description="Change the cloud origin to use">
					<CloudOriginSelect />
				</Setting>

				<Setting
					mini
					title="rspc Logger"
					description="Enable the RSPC logger so you can see what's going on in the browser logs."
				>
					<Switch
						checked={debugState.rspcLogger}
						onClick={() => (debugState.rspcLogger = !debugState.rspcLogger)}
					/>
				</Setting>
				<Setting
					mini
					title="Share telemetry"
					description="Share telemetry, even in debug mode (telemetry sharing must also be enabled in your client settings)"
				>
					<Switch
						checked={debugState.shareFullTelemetry}
						onClick={() => {
							// if debug telemetry sharing is about to be disabled, but telemetry logging is enabled
							// then disable it
							if (
								!debugState.shareFullTelemetry === false &&
								debugState.telemetryLogging
							)
								debugState.telemetryLogging = false;
							debugState.shareFullTelemetry = !debugState.shareFullTelemetry;
						}}
					/>
				</Setting>
				<Setting
					mini
					title="Telemetry logger"
					description="Enable the telemetry logger so you can see what's going on in the browser logs"
				>
					<Switch
						checked={debugState.telemetryLogging}
						onClick={() => {
							// if telemetry logging is about to be enabled, but debug telemetry sharing is disabled
							// then enable it
							if (
								!debugState.telemetryLogging &&
								debugState.shareFullTelemetry === false
							)
								debugState.shareFullTelemetry = true;
							debugState.telemetryLogging = !debugState.telemetryLogging;
						}}
					/>
				</Setting>
				{platform.openPath && (
					<Setting
						mini
						title="Open Data Directory"
						description="Quickly get to your Spacedrive database"
					>
						<div className="mt-2">
							<Button
								size="sm"
								variant="gray"
								onClick={() => {
									if (nodeState?.data?.data_path)
										platform.openPath!(nodeState?.data?.data_path);
								}}
							>
								Open
							</Button>
						</div>
					</Setting>
				)}
				{platform.reloadWebview && (
					<Setting mini title="Reload webview" description="Reload the window's webview">
						<div className="mt-2">
							<Button
								size="sm"
								variant="gray"
								onClick={() => {
									platform.reloadWebview && platform.reloadWebview();
								}}
							>
								Reload
							</Button>
						</div>
					</Setting>
				)}
				<Setting
					mini
					title="React Query Devtools"
					description="Configure the React Query devtools."
				>
					<Select
						value={debugState.reactQueryDevtools}
						size="sm"
						onChange={(value) => (debugState.reactQueryDevtools = value as any)}
					>
						<SelectOption value="disabled">Disabled</SelectOption>
						<SelectOption value="invisible">Invisible</SelectOption>
						<SelectOption value="enabled">Enabled</SelectOption>
					</Select>
				</Setting>
				<FeatureFlagSelector />
				<InvalidateDebugPanel />
				{/* <TestNotifications /> */}
				<Button size="sm" variant="gray" onClick={() => navigate('./debug/cache')}>
					Cache Debug
				</Button>
				<Button size="sm" variant="gray" onClick={() => toggleRenderRects()}>
					Toggle DND Rects
				</Button>

				{/* {platform.showDevtools && (
					<SettingContainer
						mini
						title="Devtools"
						description="Allow opening browser devtools in a production build"
					>
						<div className="mt-2">
							<Button size="sm" variant="gray" onClick={platform.showDevtools}>
								Show
							</Button>
						</div>
					</SettingContainer>
				)} */}
			</div>
		</Popover>
	);
};

function InvalidateDebugPanel() {
	const { data: count } = useBridgeQuery(['invalidation.test-invalidate']);
	const { mutate } = useLibraryMutation(['invalidation.test-invalidate-mutation']);

	return (
		<Setting
			mini
			title="Invalidate Debug Panel"
			description={`Pressing the button issues an invalidate to the query rendering this number: ${count}`}
		>
			<div className="mt-2">
				<Button size="sm" variant="gray" onClick={() => mutate(null)}>
					Invalidate
				</Button>
			</div>
		</Setting>
	);
}

function FeatureFlagSelector() {
	const featureFlags = useFeatureFlags();

	return (
		<>
			<DropdownMenu.Root
				trigger={
					<Dropdown.Button variant="gray" className="w-full">
						<span className="truncate">Feature Flags</span>
					</Dropdown.Button>
				}
				className="mt-1 shadow-none data-[side=bottom]:slide-in-from-top-2 dark:divide-menu-selected/30 dark:border-sidebar-line dark:bg-sidebar-box"
				alignToTrigger
			>
				{[...features, ...backendFeatures].map((feat) => (
					<DropdownMenu.Item
						key={feat}
						label={feat}
						iconProps={{ weight: 'bold', size: 16 }}
						onClick={() => toggleFeatureFlag(feat)}
						className="font-medium text-white"
						icon={
							featureFlags.find((f) => feat === f) !== undefined
								? CheckSquare
								: undefined
						}
					/>
				))}
			</DropdownMenu.Root>
		</>
	);
}

// function TestNotifications() {
// 	const coreNotif = useBridgeMutation(['notifications.test']);
// 	const libraryNotif = useLibraryMutation(['notifications.testLibrary']);

// 	return (
// 		<Setting mini title="Notifications" description="Test the notification system">
// 			<Button onClick={() => coreNotif.mutate(undefined)}>Core</Button>
// 			<Button onClick={() => libraryNotif.mutate(null)}>Library</Button>
// 		</Setting>
// 	);
// }

function CloudOriginSelect() {
	const origin = useBridgeQuery(['cloud.getApiOrigin']);
	const setOrigin = useBridgeMutation(['cloud.setApiOrigin']);

	const queryClient = useQueryClient();

	return (
		<>
			{origin.data && (
				<Select
					onChange={(v) =>
						setOrigin.mutateAsync(v).then(() => {
							auth.logout();
							queryClient.invalidateQueries();
						})
					}
					value={origin.data}
				>
					<SelectOption value="https://app.spacedrive.com">
						https://app.spacedrive.com
					</SelectOption>
					<SelectOption value="http://localhost:3000">http://localhost:3000</SelectOption>
				</Select>
			)}
		</>
	);
}
