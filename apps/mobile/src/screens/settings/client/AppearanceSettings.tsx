import { CheckCircle } from 'phosphor-react-native';
import React, { useState } from 'react';
import { ColorValue, Pressable, ScrollView, Text, View, ViewStyle } from 'react-native';
import { SettingsTitle } from '~/components/settings/SettingsContainer';
import Colors from '~/constants/style/Colors';
import { tw, twStyle } from '~/lib/tailwind';
import { SettingsStackScreenProps } from '~/navigation/SettingsNavigator';

type Themes = {
	insideColor: ColorValue;
	outsideColor: ColorValue;
	textColor: ColorValue;
	highlightColor: ColorValue;
	name: string;
};

// TODO: Once theming is fixed, use theme values for Light theme too.
const themes: Themes[] = [
	{
		insideColor: Colors.vanilla.app.DEFAULT,
		outsideColor: '#F0F0F0',
		textColor: Colors.vanilla.ink.DEFAULT,
		highlightColor: '#E6E6E6',
		name: 'Light'
	},
	{
		insideColor: Colors.dark.app.DEFAULT,
		outsideColor: Colors.dark.app.darkBox,
		textColor: Colors.dark.ink.DEFAULT,
		highlightColor: Colors.dark.app.line,
		name: 'Dark'
	},
	{
		insideColor: '#000000',
		outsideColor: '#000000',
		textColor: '#000000',
		highlightColor: '#000000',
		name: 'System'
	}
];

type ThemeProps = Themes & { isSelected?: boolean; containerStyle?: ViewStyle };

function Theme(props: ThemeProps) {
	return (
		<View style={props.containerStyle}>
			<View
				style={twStyle(
					{ backgroundColor: props.outsideColor, borderColor: props.highlightColor },
					'relative h-[80px] w-[100px] overflow-hidden rounded-xl border-2 border-app-line',
					props.isSelected && 'border-white'
				)}
			>
				<View
					style={twStyle(
						{ backgroundColor: props.insideColor, borderColor: props.highlightColor },
						'absolute bottom-[-2px] right-[-2px] h-[60px] w-[75px] rounded-tl-xl border'
					)}
				>
					<Text
						style={twStyle({ color: props.textColor }, 'ml-3 mt-1 text-lg font-medium')}
					>
						Aa
					</Text>
				</View>
				{/* Checkmark */}
				{props.isSelected && (
					<CheckCircle
						color={props.textColor as string}
						weight="fill"
						size={24}
						style={tw`absolute right-1.5 bottom-1.5`}
					/>
				)}
			</View>
		</View>
	);
}

function SystemTheme(props: { isSelected: boolean }) {
	return (
		<View style={tw`h-[90px] w-[110px] flex-1 flex-row overflow-hidden rounded-xl`}>
			<View style={twStyle('flex-1', { backgroundColor: themes[1]?.outsideColor })}></View>
			<View style={twStyle('flex-1', { backgroundColor: themes[0]?.outsideColor })}></View>
		</View>
	);
}

const AppearanceSettingsScreen = ({
	navigation
}: SettingsStackScreenProps<'AppearanceSettings'>) => {
	const [selectedTheme, setSelectedTheme] = useState(themes[2]?.name);
	return (
		<View style={tw`flex-1 pt-4`}>
			<View style={tw`px-4`}>
				<SettingsTitle>Theme</SettingsTitle>
				<View style={tw`mb-4 border-b border-b-app-line`} />
				<ScrollView
					horizontal
					showsHorizontalScrollIndicator={false}
					contentContainerStyle={tw`gap-x-2`}
				>
					{themes.map((theme) => (
						<Pressable key={theme.name} onPress={() => setSelectedTheme(theme.name)}>
							{theme.name === 'System' ? (
								<SystemTheme isSelected={selectedTheme === 'System'} />
							) : (
								<Theme {...theme} isSelected={selectedTheme === theme.name} />
							)}
							<Text style={tw`mt-1.5 text-center font-medium text-white`}>
								{theme.name}
							</Text>
						</Pressable>
					))}
				</ScrollView>
			</View>
		</View>
	);
};

export default AppearanceSettingsScreen;
