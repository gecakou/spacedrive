// import types from '../../constants/file-types.json';
import { useLibraryQuery } from '@sd/client';
import { ExplorerContext, ExplorerItem } from '@sd/client';
import { Button } from '@sd/ui';
import { useQuery } from '@tanstack/react-query';
import clsx from 'clsx';
import dayjs from 'dayjs';
import { Link, Share } from 'phosphor-react';
import { useEffect, useState } from 'react';

import { DefaultProps } from '../primitive/types';
import { Tooltip } from '../tooltip/Tooltip';
import FileThumb from './FileThumb';
import { Divider } from './inspector/Divider';
import FavoriteButton from './inspector/FavoriteButton';
import { MetaItem } from './inspector/MetaItem';
import Note from './inspector/Note';
import { isObject } from './utils';

interface Props extends DefaultProps<HTMLDivElement> {
	context?: ExplorerContext;
	data?: ExplorerItem;
}

export const Inspector = (props: Props) => {
	const { context, data, ...elementProps } = props;

	const { data: types } = useQuery(
		['_file-types'],
		() => import('../../constants/file-types.json')
	);

	const is_dir = props.data?.type === 'Path' ? props.data.is_dir : false;

	const objectData = props.data ? (isObject(props.data) ? props.data : props.data.object) : null;

	// this prevents the inspector from fetching data when the user is navigating quickly
	const [readyToFetch, setReadyToFetch] = useState(false);
	useEffect(() => {
		const timeout = setTimeout(() => {
			setReadyToFetch(true);
		}, 350);
		return () => clearTimeout(timeout);
	}, [props.data?.id]);

	// this is causing LAG
	const tags = useLibraryQuery(['tags.getForObject', objectData?.id || -1], {
		enabled: readyToFetch
	});

	const isVid = isVideo(props.data?.extension || '');

	return (
		<div
			{...elementProps}
			className="-mt-[50px] pt-[55px] pl-1.5 pr-1 w-full h-screen overflow-x-hidden custom-scroll inspector-scroll pb-[55px]"
		>
			{!!props.data && (
				<>
					<div className="flex bg-sidebar items-center justify-center w-full h-64 mb-[10px] overflow-hidden rounded-lg ">
						<FileThumb
							iconClassNames="mx-10"
							size={230}
							kind={props.data.extension === 'zip' ? 'zip' : isVid ? 'video' : 'other'}
							className="!m-0 flex bg-green-500 flex-shrink flex-grow-0"
							data={props.data}
						/>
					</div>
					<div className="flex flex-col w-full pt-0.5 pb-1 overflow-hidden bg-app-box rounded-lg select-text shadow-app-shade/10 border border-app-line">
						<h3 className="pt-2 pb-1 pl-3 text-base font-bold">
							{props.data?.name}
							{props.data?.extension && `.${props.data.extension}`}
						</h3>
						{objectData && (
							<div className="flex flex-row mt-1 mx-3 space-x-0.5">
								<Tooltip label="Favorite">
									<FavoriteButton data={objectData} />
								</Tooltip>
								<Tooltip label="Share">
									<Button size="icon">
										<Share className="w-[18px] h-[18px]" />
									</Button>
								</Tooltip>
								<Tooltip label="Link">
									<Button size="icon">
										<Link className="w-[18px] h-[18px]" />
									</Button>
								</Tooltip>
							</div>
						)}
						{tags?.data && tags.data.length > 0 && (
							<>
								<Divider />
								<MetaItem
									value={
										<div className="flex flex-wrap  gap-1.5">
											{tags?.data?.map((tag) => (
												<div
													// onClick={() => setSelectedTag(tag.id === selectedTag ? null : tag.id)}
													key={tag.id}
													className={clsx(
														'flex items-center rounded px-1.5 py-0.5'
														// selectedTag === tag.id && 'ring'
													)}
													style={{ backgroundColor: tag.color + 'CC' }}
												>
													<span className="text-xs text-white drop-shadow-md">{tag.name}</span>
												</div>
											))}
										</div>
									}
								/>
							</>
						)}
						{props.context?.type == 'Location' && props.data?.type === 'Path' && (
							<>
								<Divider />
								<MetaItem
									title="URI"
									value={`${props.context.local_path}/${props.data.materialized_path}`}
								/>
							</>
						)}
						<Divider />
						<MetaItem
							title="Date Created"
							value={dayjs(props.data?.date_created).format('MMMM Do YYYY, h:mm:ss a')}
						/>
						<Divider />
						<MetaItem
							title="Date Indexed"
							value={dayjs(props.data?.date_indexed).format('MMMM Do YYYY, h:mm:ss a')}
						/>
						{!is_dir && (
							<>
								<Divider />
								<div className="flex flex-row items-center px-3 py-2 meta-item">
									{props.data?.extension && (
										<span className="inline px-1 mr-1 text-xs font-bold uppercase bg-gray-500 rounded-md text-gray-150">
											{props.data?.extension}
										</span>
									)}
									<p className="text-xs text-gray-600 break-all truncate dark:text-gray-300">
										{props.data?.extension
											? //@ts-ignore
											  types[props.data.extension.toUpperCase()]?.descriptions.join(' / ')
											: 'Unknown'}
									</p>
								</div>
								{objectData && (
									<>
										<Note data={objectData} />
										<Divider />
										{objectData.cas_id && (
											<MetaItem title="Unique Content ID" value={objectData.cas_id} />
										)}
									</>
								)}
							</>
						)}
					</div>
				</>
			)}
		</div>
	);
};

function isVideo(extension: string) {
	return [
		'avi',
		'asf',
		'mpeg',
		'mts',
		'mpe',
		'vob',
		'qt',
		'mov',
		'asf',
		'asx',
		'mjpeg',
		'ts',
		'mxf',
		'm2ts',
		'f4v',
		'wm',
		'3gp',
		'm4v',
		'wmv',
		'mp4',
		'webm',
		'flv',
		'mpg',
		'hevc',
		'ogv',
		'swf',
		'wtv'
	].includes(extension);
}
