import { AppLogo } from '@sd/assets/images';
import { ButtonLink } from '@sd/ui';
import { OnboardingContainer, OnboardingDescription, OnboardingImg } from './Layout';
import { useUnlockOnboardingScreen } from './Progress';

export default function OnboardingStart() {
	useUnlockOnboardingScreen();
	return (
		<OnboardingContainer>
			<OnboardingImg src={AppLogo} className="mb-8 h-36 w-36 shrink-0" />

			<h1 className="mb-2 text-center text-4xl font-bold text-ink">
				The file explorer from the future.
			</h1>
			<OnboardingDescription>
				Welcome to Spacedrive, an open source cross-platform file manager.
			</OnboardingDescription>
			<div className="mt-6 space-x-3">
				<ButtonLink to="/onboarding/new-library" replace variant="accent" size="md">
					Get started
				</ButtonLink>
			</div>
		</OnboardingContainer>
	);
}
