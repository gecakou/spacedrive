import React from 'react';

import { useNodeStore } from '../../components/device/Stores';
import { Toggle } from '../../../../ui/src/Toggle';
import { InputContainer } from '../../../../ui/src/InputContainer';
import { SettingsContainer } from '../../components/settings/SettingsContainer';
import { SettingsHeader } from '../../components/settings/SettingsHeader';

export default function ExperimentalSettings() {
	// const locations = useBridgeQuery("SysGetLocation")

	const { isExperimental, setIsExperimental } = useNodeStore();

	return (
		<SettingsContainer>
			{/*<Button size="sm">Add Location</Button>*/}
			<SettingsHeader title="Experimental" description="Experimental features within Spacedrive." />
			<InputContainer
				mini
				title="Debug Menu"
				description="Shows data about Spacedrive such as Jobs, Job History and Client State."
			>
				<div className="flex items-center h-full pl-10">
					<Toggle
						value={isExperimental}
						size={'sm'}
						onChange={(newValue) => {
							setIsExperimental(!isExperimental);
						}}
					/>
				</div>
			</InputContainer>
		</SettingsContainer>
	);
}
