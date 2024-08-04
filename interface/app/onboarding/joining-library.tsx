import { Loader } from '@sd/ui';

import { OnboardingContainer, OnboardingDescription, OnboardingTitle } from './components';

export default function OnboardingCreatingLibrary() {
	return (
		<OnboardingContainer>
			<span className="text-6xl">🛠</span>
			<OnboardingTitle>Joining library</OnboardingTitle>
			<OnboardingDescription>Joining library...</OnboardingDescription>
			<Loader className="mt-5" />
		</OnboardingContainer>
	);
}
