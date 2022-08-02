import { FilePath } from '@sd/core';
import { Cloud, Desktop, DeviceMobileCamera, Laptop } from 'phosphor-react-native';
import React from 'react';
import { FlatList, Text, View } from 'react-native';
import { LockClosedIcon } from 'react-native-heroicons/solid';

import tw from '../../lib/tailwind';
import FileItem from '../file/FileItem';

export interface DeviceProps {
	name: string;
	size: string;
	type: 'laptop' | 'desktop' | 'phone' | 'server';
	locations: Array<{ name: string; folder?: boolean; format?: string; icon?: string }>;
}

const placeholderFileItems: FilePath[] = [
	{
		is_dir: true,
		date_created: '2020-01-01T00:00:00.000Z',
		date_indexed: '2020-01-01T00:00:00.000Z',
		date_modified: '2020-01-01T00:00:00.000Z',
		extension: '',
		file_id: 1,
		id: 1,
		location_id: 1,
		materialized_path: '',
		name: 'Documents',
		parent_id: 0,
		file: {
			id: 1,
			cas_id: '',
			ipfs_id: '',
			has_thumbnail: false,
			favorite: false,
			has_thumbstrip: false,
			has_video_preview: false,
			hidden: false,
			important: false,
			integrity_checksum: '',
			kind: 'Image',
			note: '',
			paths: [],
			size_in_bytes: '555',
			date_created: '',
			date_indexed: '',
			date_modified: ''
		}
	},
	{
		is_dir: true,
		date_created: '2020-01-01T00:00:00.000Z',
		date_indexed: '2020-01-01T00:00:00.000Z',
		date_modified: '2020-01-01T00:00:00.000Z',
		extension: '',
		file_id: 2,
		id: 2,
		location_id: 2,
		materialized_path: '',
		name: 'Movies',
		parent_id: 0,
		file: {
			id: 2,
			cas_id: '',
			ipfs_id: '',
			has_thumbnail: false,
			favorite: false,
			has_thumbstrip: false,
			has_video_preview: false,
			hidden: false,
			important: false,
			integrity_checksum: '',
			kind: 'Image',
			note: '',
			paths: [],
			size_in_bytes: '555',
			date_created: '',
			date_indexed: '',
			date_modified: ''
		}
	},
	{
		is_dir: true,
		date_created: '2020-01-01T00:00:00.000Z',
		date_indexed: '2020-01-01T00:00:00.000Z',
		date_modified: '2020-01-01T00:00:00.000Z',
		extension: '',
		file_id: 3,
		id: 3,
		location_id: 3,
		materialized_path: '',
		name: 'Minecraft',
		parent_id: 0,
		file: {
			id: 3,
			cas_id: '',
			ipfs_id: '',
			has_thumbnail: false,
			favorite: false,
			has_thumbstrip: false,
			has_video_preview: false,
			hidden: false,
			important: false,
			integrity_checksum: '',
			kind: 'Image',
			note: '',
			paths: [],
			size_in_bytes: '555',
			date_created: '',
			date_indexed: '',
			date_modified: ''
		}
	}
];

const Device = ({ name, locations, size, type }: DeviceProps) => {
	return (
		<View style={tw`bg-gray-600 border rounded-md border-gray-550 mt-4`}>
			<View style={tw`flex flex-row items-center px-4 pt-3 pb-2`}>
				<View style={tw`flex flex-row items-center`}>
					{type === 'phone' && (
						<DeviceMobileCamera color="white" weight="fill" size={18} style={tw`mr-2`} />
					)}
					{type === 'laptop' && <Laptop color="white" weight="fill" size={18} style={tw`mr-2`} />}
					{type === 'desktop' && <Desktop color="white" weight="fill" size={18} style={tw`mr-2`} />}
					{type === 'server' && <Cloud color="white" weight="fill" size={18} style={tw`mr-2`} />}
					<Text style={tw`text-base font-semibold text-white`}>{name || 'Unnamed Device'}</Text>
					{/* P2P Lock */}
					<View style={tw`flex flex-row rounded items-center ml-2 bg-gray-500 py-[1px] px-[4px]`}>
						<LockClosedIcon size={12} color={tw.color('gray-400')} />
						<Text style={tw`text-gray-400 font-semibold ml-0.5 text-xs`}>P2P</Text>
					</View>
				</View>
				{/* Size */}
				<Text style={tw`font-semibold text-sm ml-2 text-gray-400`}>{size}</Text>
			</View>
			{/* Locations/Files TODO: Maybe use FlashList? */}
			<FlatList
				data={placeholderFileItems}
				renderItem={({ item }) => <FileItem file={item} />}
				keyExtractor={(item) => item.id.toString()}
				horizontal
				contentContainerStyle={tw`mt-4 ml-2`}
			/>
		</View>
	);
};

export default Device;
