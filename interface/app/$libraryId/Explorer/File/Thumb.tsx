import * as icons from '@sd/assets/icons';
import clsx from 'clsx';
import { CSSProperties } from 'react';
import { ExplorerItem } from '@sd/client';
import { useIsDark, usePlatform } from '~/util/Platform';
import { getExplorerItemData } from '../util';
import classes from './Thumb.module.scss';

interface Props {
	data: ExplorerItem;
	size: number;
	className?: string;
}

export default ({ data, size, className }: Props) => {
	const { cas_id, isDir, kind, hasThumbnail, extension } = getExplorerItemData(data);

	// 10 percent of the size
	const videoBarsHeight = Math.floor(size / 10);

	// calculate 16:9 ratio for height from size
	const videoHeight = Math.floor((size * 9) / 16) + videoBarsHeight * 2;

	return (
		<div
			className={clsx(
				'relative flex h-full shrink-0 items-center justify-center border-2 border-transparent',
				className
			)}
		>
			<FileThumbImg
				size={size}
				hasThumbnail={hasThumbnail}
				isDir={isDir}
				cas_id={cas_id}
				extension={extension}
				kind={kind}
				imgClassName={clsx(
					hasThumbnail &&
						'max-h-full w-auto max-w-full rounded-sm object-cover shadow shadow-black/30',
					kind === 'Image' && classes.checkers,
					kind === 'Image' && size > 60 && 'border-app-line border-2',
					kind === 'Video' && 'rounded border-x-0 !border-black'
				)}
				imgStyle={
					kind === 'Video'
						? {
								borderTopWidth: videoBarsHeight,
								borderBottomWidth: videoBarsHeight,
								width: size,
								height: videoHeight
						  }
						: {}
				}
			/>
			{extension && kind === 'Video' && hasThumbnail && size > 80 && (
				<div className="absolute bottom-[13%] right-[5%] rounded bg-black/60 py-0.5 px-1 text-[9px] font-semibold uppercase opacity-70">
					{extension}
				</div>
			)}
		</div>
	);
};
interface FileThumbImgProps {
	isDir: boolean;
	cas_id: string | null;
	kind: string | null;
	extension: string | null;
	size: number;
	hasThumbnail: boolean;
	imgClassName?: string;
	imgStyle?: CSSProperties;
}

export function FileThumbImg({
	isDir,
	cas_id,
	kind,
	size,
	hasThumbnail,
	extension,
	imgClassName,
	imgStyle
}: FileThumbImgProps) {
	const platform = usePlatform();

	// is dark mode
	const isDark = useIsDark();

	if (hasThumbnail && cas_id) {
		return (
			<img
				style={{ ...imgStyle, maxWidth: size, width: size - 10 }}
				decoding="async"
				className={clsx('z-90 pointer-events-none', imgClassName)}
				src={platform.getThumbnailUrlById(cas_id)}
			/>
		);
	}

	// Render an img component with an image based on kind
	let icon = icons['Document'];

	if (isDir) {
		icon = icons['Folder'];
	} else if (
		kind &&
		extension &&
		icons[`${kind}_${extension.toLowerCase()}` as keyof typeof icons]
	) {
		icon = icons[`${kind}_${extension.toLowerCase()}` as keyof typeof icons];
	} else if (kind !== 'Unknown' && kind && icons[kind as keyof typeof icons]) {
		icon = icons[kind as keyof typeof icons];
	}

	if (!isDark) icon = icon?.substring(0, icon.length - 4) + '_Light' + '.png';

	return <img src={icon} className={clsx('h-full overflow-hidden')} />;
}
