import { useState } from 'react';

import { Toggle } from '../../../components/primitive';
import { InputContainer } from '../../../components/primitive/InputContainer';
import { SettingsContainer } from '../../../components/settings/SettingsContainer';
import { SettingsHeader } from '../../../components/settings/SettingsHeader';

export default function AppearanceSettings() {
	const [syncWithLibrary, setSyncWithLibrary] = useState(true);
	return (
		<SettingsContainer>
			<SettingsHeader title="Keybindings" description="Manage client keybindings" />
			<InputContainer
				mini
				title="Sync with Library"
				description="If enabled your keybindings will be synced with library, otherwise they will apply only to this client."
			>
				<Toggle value={syncWithLibrary} onChange={setSyncWithLibrary} className="m-2 ml-4" />
			</InputContainer>
		</SettingsContainer>
	);
}
