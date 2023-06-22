import * as icons from '@sd/assets/icons';
import { PropsWithChildren } from 'react';
import { Image, View } from 'react-native';
import { DocumentDirectoryPath } from 'react-native-fs';
import { ExplorerItem, ObjectKind, getItemFilePath, getItemObject, isPath } from '@sd/client';
import { tw } from '../../lib/tailwind';
import FolderIcon from '../icons/FolderIcon';

type FileThumbProps = {
	data: ExplorerItem;
	/**
	 * This is multiplier for calculating icon size
	 * default: `1`
	 */
	size?: number;
};

export const getThumbnailUrlById = (keyParts: string[]) =>
	`${DocumentDirectoryPath}/thumbnails/${keyParts
		.map((i) => encodeURIComponent(i))
		.join('/')}.webp`;

type KindType = keyof typeof icons | 'Unknown';

function getExplorerItemData(data: ExplorerItem) {
	const objectData = getItemObject(data);
	const filePath = getItemFilePath(data);

	return {
		casId: filePath?.cas_id || null,
		isDir: isPath(data) && data.item.is_dir,
		kind: ObjectKind[objectData?.kind || 0] as KindType,
		hasLocalThumbnail: data.has_local_thumbnail, // this will be overwritten if new thumbnail is generated
		thumbnailKey: data.thumbnail_key,
		extension: filePath?.extension
	};
}

const FileThumbWrapper = ({ children, size = 1 }: PropsWithChildren<{ size: number }>) => (
	<View style={[tw`items-center justify-center`, { width: 80 * size, height: 80 * size }]}>
		{children}
	</View>
);

export default function FileThumb({ data, size = 1 }: FileThumbProps) {
	const { casId, isDir, kind, hasLocalThumbnail, extension, thumbnailKey } =
		getExplorerItemData(data);

	if (isPath(data) && data.item.is_dir) {
		return (
			<FileThumbWrapper size={size}>
				<FolderIcon size={70 * size} />
			</FileThumbWrapper>
		);
	}

	if (hasLocalThumbnail && thumbnailKey) {
		// TODO: Handle Image checkers bg?
		return (
			<FileThumbWrapper size={size}>
				<Image
					source={{ uri: getThumbnailUrlById(thumbnailKey) }}
					resizeMode="contain"
					style={tw`h-full w-full`}
				/>
			</FileThumbWrapper>
		);
	}

	// Default icon
	let icon = icons['Document'];

	if (isDir) {
		icon = icons['Folder'];
	} else if (
		kind &&
		extension &&
		icons[`${kind}_${extension.toLowerCase()}` as keyof typeof icons]
	) {
		// e.g. Document_pdf
		icon = icons[`${kind}_${extension.toLowerCase()}` as keyof typeof icons];
	} else if (kind !== 'Unknown' && kind && icons[kind]) {
		icon = icons[kind];
	}

	// TODO: Handle video thumbnails (do we have ffmpeg on mobile?)

	// // 10 percent of the size
	// const videoBarsHeight = Math.floor(size / 10);

	// // calculate 16:9 ratio for height from size
	// const videoHeight = Math.floor((size * 9) / 16) + videoBarsHeight * 2;

	return (
		<FileThumbWrapper size={size}>
			<Image source={icon} style={{ width: 70 * size, height: 70 * size }} />
		</FileThumbWrapper>
	);
}
