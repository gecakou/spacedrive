import clsx from 'clsx';
import { CheckCircle } from 'phosphor-react';
import { useEffect } from 'react';
import { useState } from 'react';
import { getThemeStore, useThemeStore } from '@sd/client';
import { Themes } from '@sd/client';
import { forms } from '@sd/ui';
import { usePlatform } from '~/util/Platform';
import { Heading } from '../Layout';
import Setting from '../Setting';

type Theme = {
	insideColor: string;
	outsideColor: string;
	textColor: string;
	border: string;
	themeName: string;
	themeValue: Themes | 'system';
};

type ThemeProps = Theme & { isSelected?: boolean; className?: string };

const { Form, Switch, useZodForm, z } = forms;

const schema = z.object({
	uiAnimations: z.boolean(),
	syncThemeWithSystem: z.boolean(),
	blurEffects: z.boolean()
});

const themes: Theme[] = [
	{
		insideColor: 'bg-white',
		outsideColor: 'bg-[#F0F0F0]',
		textColor: 'text-black',
		border: 'border border-[#E6E6E6]',
		themeName: 'Light',
		themeValue: 'vanilla'
	},
	{
		insideColor: 'bg-[#1C1D26]', //Not using theme color because we want it to stay the same color when theme is toggled
		outsideColor: 'bg-black',
		textColor: 'text-white',
		border: 'border border-[#323342]',
		themeName: 'Dark',
		themeValue: 'dark'
	},
	{
		insideColor: '',
		outsideColor: '',
		textColor: 'text-white',
		border: 'border border-[#323342]',
		themeName: 'System',
		themeValue: 'system'
	}
];

export const Component = () => {
	const themeStore = useThemeStore();
	const platform = usePlatform();
	const [selectedTheme, setSelectedTheme] = useState<Theme['themeValue']>(
		themeStore.syncThemeWithSystem === true ? 'system' : themeStore.theme
	);
	const form = useZodForm({
		schema
	});

	const onSubmit = form.handleSubmit(async (data) => {
		console.log({ data });
	});

	useEffect(() => {
		const subscription = form.watch(() => onSubmit());
		return () => subscription.unsubscribe();
	}, [form, onSubmit]);

	const themeSelectHandler = (theme: Theme['themeValue']) => {
		setSelectedTheme(theme);
		if (theme === 'system') {
			getThemeStore().syncThemeWithSystem = true;
		} else if (theme === 'vanilla') {
			getThemeStore().syncThemeWithSystem = false;
			getThemeStore().theme = theme;
			document.documentElement.classList.add('vanilla-theme');
		} else if (theme === 'dark') {
			getThemeStore().syncThemeWithSystem = false;
			getThemeStore().theme = theme;
			document.documentElement.classList.remove('vanilla-theme');
		}
	};

	return (
		<Form form={form} onSubmit={onSubmit}>
			<Heading title="Appearance" description="Change the look of your client." />
			<div className="mb-14 mt-8 flex h-[90px] w-full flex-wrap gap-5">
				{themes.map((theme, i) => {
					return (
						<div
							onClick={() => themeSelectHandler(theme.themeValue)}
							className={clsx(
								selectedTheme !== theme.themeValue && 'opacity-70',
								'transition-all duration-200 hover:translate-y-[-3.5px]'
							)}
							key={i}
						>
							{theme.themeValue === 'system' ? (
								<SystemTheme {...theme} isSelected={selectedTheme === 'system'} />
							) : (
								<Theme {...theme} isSelected={selectedTheme === theme.themeValue} />
							)}
						</div>
					);
				})}
			</div>
			<Setting
				mini
				title="Sync Theme with System"
				description="The theme of the client will change based on your system theme."
			>
				<Switch disabled {...form.register('syncThemeWithSystem')} className="m-2 ml-4" />
			</Setting>
			<Setting
				mini
				title="UI Animations"
				description="Dialogs and other UI elements will animate when opening and closing."
			>
				<Switch disabled {...form.register('uiAnimations')} className="m-2 ml-4" />
			</Setting>
			<Setting
				mini
				title="Blur Effects"
				description="Some components will have a blur effect applied to them."
			>
				<Switch disabled {...form.register('blurEffects')} className="m-2 ml-4" />
			</Setting>
		</Form>
	);
};

function Theme(props: ThemeProps) {
	return (
		<div className="h-full">
			<div
				className={clsx(
					props.outsideColor,
					props.border,
					props.textColor,
					props.className,
					'relative h-full w-[150px] overflow-hidden rounded-lg'
				)}
			>
				<div
					className={clsx(
						props.insideColor,
						props.border,
						'absolute bottom-[-10px] right-[-2px] h-[70px] w-[118px] rounded-tl-lg p-3'
					)}
				>
					<p>Aa</p>
				</div>
				{props.isSelected && (
					<CheckCircle
						weight="fill"
						size={24}
						className={`absolute bottom-1.5 right-1.5 z-10 text-accent`}
					/>
				)}
			</div>
			<p className="mt-3 text-center text-sm">{props.themeName}</p>
		</div>
	);
}

function SystemTheme(props: ThemeProps) {
	return (
		<div className="h-full w-[150px]">
			<div className="relative flex h-full">
				<div className="relative h-full w-[50%] grow overflow-hidden rounded-bl-lg rounded-tl-lg bg-black">
					<Theme className="rounded-br-none rounded-tr-none" {...themes[1]!} />
				</div>
				<div
					className={clsx(
						'relative h-full w-[50%] grow overflow-hidden rounded-br-lg rounded-tr-lg'
					)}
				>
					<Theme className="rounded-bl-none rounded-tl-none" {...themes[0]!} />
				</div>
				{props.isSelected && (
					<CheckCircle
						weight="fill"
						size={24}
						className={`absolute bottom-1.5 right-1.5 z-10 text-accent`}
					/>
				)}
			</div>
			<p className="mt-3 text-center text-sm">{props.themeName}</p>
		</div>
	);
}
