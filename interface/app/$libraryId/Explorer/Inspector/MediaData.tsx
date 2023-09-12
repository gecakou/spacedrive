import {
	CoordinatesFormat,
	MediaLocation,
	MediaMetadata,
	MediaTime,
	useThemeStore
} from '@sd/client';
import Accordion from '~/components/Accordion';
import { usePlatform } from '~/util/Platform';

import { MetaData } from './index';

interface Props {
	data: MediaMetadata;
}

const formatMediaTime = (time: MediaTime): string | null => {
	if (time === 'Undefined') return null;
	if ('Utc' in time) return time.Utc;
	if ('Naive' in time) return time.Naive;
	return null;
};

const formatLocationDD = (loc: MediaLocation, dp: number): string => {
	// the lack of a + here will mean that coordinates may have padding at the end
	// google does the same (if one is larger than the other, the smaller one will be padded with zeroes)
	return `${loc.latitude.toFixed(dp)}, ${loc.longitude.toFixed(dp)}`;
};

const formatLocationDMS = (loc: MediaLocation): string => {
	const formatCoordinatesAsDMS = (
		coordinates: number,
		positiveChar: string,
		negativeChar: string
	): string => {
		const abs = getAbsoluteDecimals(coordinates);
		const d = Math.trunc(coordinates);
		const m = Math.trunc(60 * abs);
		// adding 0.05 before rounding and truncating with `toFixed` makes it match up with google
		const s = (abs * 3600 - m * 60 + 0.05).toFixed(1);
		const sign = coordinates > 0 ? positiveChar : negativeChar;
		return `${d}°${m}'${s}"${sign}`;
	};

	return `${formatCoordinatesAsDMS(loc.latitude, 'N', 'S')} ${formatCoordinatesAsDMS(
		loc.longitude,
		'E',
		'W'
	)}`;
};

const getAbsoluteDecimals = (num: number): number => {
	const x = num.toString();
	// this becomes +0.xxxxxxxxx and is needed to convert the minutes/seconds for DMS
	return Math.abs(Number.parseFloat('0.' + x.substring(x.indexOf('.') + 1)));
};

const formatLocation = (loc: MediaLocation, format: CoordinatesFormat, dp?: number): string => {
	return format === 'dd' && dp ? formatLocationDD(loc, dp) : formatLocationDMS(loc);
};

const orientations = {
	Normal: 'Normal',
	MirroredHorizontal: 'Horizontally mirrored',
	MirroredHorizontalAnd90CW: 'Mirrored horizontally and rotated 90° clockwise',
	MirroredHorizontalAnd270CW: 'Mirrored horizontally and rotated 270° clockwise',
	MirroredVertical: 'Vertically mirrored',
	CW90: 'Rotated 90° clockwise',
	CW180: 'Rotated 180° clockwise',
	CW270: 'Rotated 270° clockwise'
};

const MediaData = ({ data }: Props) => {
	const platform = usePlatform();
	const coordinatesFormat = useThemeStore().coordinatesFormat;

	return data.type === 'Image' ? (
		<div className="flex flex-col gap-0 py-2">
			<Accordion variant="apple" title="More info">
				<MetaData label="Date" value={formatMediaTime(data.date_taken)} />
				<MetaData label="Type" value={data.type} />
				<MetaData
					label="Location"
					tooltipValue={
						data.location ? formatLocation(data.location, coordinatesFormat, 8) : '--'
					}
					value={
						data.location ? (
							<a
								onClick={(e) => {
									e.preventDefault();
									if (data.location)
										platform.openLink(
											`https://www.google.com/maps/search/?api=1&query=${encodeURIComponent(
												`${data.location.latitude}, ${data.location.longitude}`
											)}`
										);
								}}
							>
								{formatLocation(data.location, coordinatesFormat, 4)}
							</a>
						) : (
							'--'
						)
					}
				/>
				<MetaData
					label="Plus Code"
					value={
						data.location?.pluscode ? (
							<a
								onClick={(e) => {
									e.preventDefault();
									if (data.location)
										platform.openLink(
											`https://www.google.com/maps/search/?api=1&query=${encodeURIComponent(
												data.location.pluscode
											)}`
										);
								}}
							>
								{data.location?.pluscode}
							</a>
						) : (
							'--'
						)
					}
				/>
				<MetaData
					label="Dimensions"
					value={
						<>
							{data.dimensions.width} x {data.dimensions.height}
						</>
					}
				/>
				<MetaData label="Device" value={data.camera_data.device_make} />
				<MetaData label="Model" value={data.camera_data.device_model} />
				<MetaData
					label="Orientation"
					value={orientations[data.camera_data.orientation] ?? '--'}
				/>
				<MetaData label="Color profile" value={data.camera_data.color_profile} />
				<MetaData label="Color space" value={data.camera_data.color_space} />
				<MetaData label="Flash" value={data.camera_data.flash?.mode} />
				<MetaData label="Zoom" value={data.camera_data.zoom?.toFixed(2)} />
				<MetaData label="Iso" value={data.camera_data.iso} />
				<MetaData label="Software" value={data.camera_data.software} />
			</Accordion>
		</div>
	) : null;
};

export default MediaData;
