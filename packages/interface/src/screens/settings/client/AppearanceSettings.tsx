import { useState } from 'react';
import { Switch } from '@sd/ui';
import { InputContainer } from '~/components/primitive/InputContainer';
import { SettingsContainer } from '~/components/settings/SettingsContainer';
import { SettingsHeader } from '~/components/settings/SettingsHeader';

export default function AppearanceSettings() {
	const [uiAnimations, setUiAnimations] = useState(true);
	const [blurEffects, setBlurEffects] = useState(true);

	return (
		<SettingsContainer>
			<SettingsHeader title="Appearance" description="Change the look of your client." />
			<InputContainer
				mini
				title="UI Animations"
				description="Dialogs and other UI elements will animate when opening and closing."
			>
				<Switch checked={uiAnimations} onCheckedChange={setUiAnimations} className="m-2 ml-4" />
			</InputContainer>
			<InputContainer
				mini
				title="Blur Effects"
				description="Some components will have a blur effect applied to them."
			>
				<Switch checked={blurEffects} onCheckedChange={setBlurEffects} className="m-2 ml-4" />
			</InputContainer>
		</SettingsContainer>
	);
}
