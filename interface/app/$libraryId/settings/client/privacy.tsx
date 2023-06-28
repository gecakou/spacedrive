import { telemetryStore, useTelemetryState } from '@sd/client';
import { Switch } from '@sd/ui';
import { Heading } from '../Layout';
import Setting from '../Setting';

export const Component = () => {
	const shareTelemetry = useTelemetryState().shareTelemetry;

	return (
		<>
			<Heading title="Privacy" description="" />
			<Setting
				mini
				title="Share Telemetry and Usage Data"
				description="Share anonymous usage data to help us improve the app."
			>
				<Switch
					checked={shareTelemetry}
					onClick={() => (telemetryStore.shareTelemetry = !shareTelemetry)}
					className="m-2 ml-4"
					size="md"
				/>
			</Setting>
		</>
	);
};
