import * as icons from '@sd/assets/icons';
import byteSize from 'byte-size';
import clsx from 'clsx';
import { useMemo, useState } from 'react';
import Skeleton from 'react-loading-skeleton';
import 'react-loading-skeleton/dist/skeleton.css';
import {
	ExplorerItem,
	ObjectKind,
	ObjectKindKey,
	Statistics,
	useLibraryContext,
	useLibraryQuery
} from '@sd/client';
import { z } from '@sd/ui/src/forms';
import { useExplorerStore, useExplorerTopBarOptions } from '~/hooks';
import useCounter from '~/hooks/useCounter';
import { usePlatform } from '~/util/Platform';
import Explorer from './Explorer';
import { SEARCH_PARAMS, getExplorerItemData } from './Explorer/util';
import { usePageLayout } from './PageLayout';
import { ToolOption } from './TopBar';
import TopBarChildren from './TopBar/TopBarChildren';

interface StatItemProps {
	title: string;
	bytes: bigint;
	isLoading: boolean;
}

const StatItemNames: Partial<Record<keyof Statistics, string>> = {
	total_bytes_capacity: 'Total capacity',
	preview_media_bytes: 'Preview media',
	library_db_size: 'Index size',
	total_bytes_free: 'Free space'
};

const EMPTY_STATISTICS = {
	id: 0,
	date_captured: '',
	total_bytes_capacity: '0',
	preview_media_bytes: '0',
	library_db_size: '0',
	total_object_count: 0,
	total_bytes_free: '0',
	total_bytes_used: '0',
	total_unique_bytes: '0'
};

const displayableStatItems = Object.keys(StatItemNames) as unknown as keyof typeof StatItemNames;

let overviewMounted = false;

const StatItem = (props: StatItemProps) => {
	const { title, bytes = BigInt('0'), isLoading } = props;

	const size = byteSize(Number(bytes)); // TODO: This BigInt to Number conversion will truncate the number if the number is too large. `byteSize` doesn't support BigInt so we are gonna need to come up with a longer term solution at some point.
	const count = useCounter({
		name: title,
		end: +size.value,
		duration: overviewMounted ? 0 : 1,
		saveState: false
	});

	return (
		<div
			className={clsx(
				'flex w-32 shrink-0 cursor-default flex-col rounded-md px-4 py-3 duration-75',
				!bytes && 'hidden'
			)}
		>
			<span className="text-sm text-gray-400">{title}</span>
			<span className="text-2xl">
				{isLoading && (
					<div>
						<Skeleton
							enableAnimation={true}
							baseColor={'#21212e'}
							highlightColor={'#13131a'}
						/>
					</div>
				)}
				<div
					className={clsx({
						hidden: isLoading
					})}
				>
					<span className="font-black tabular-nums">{count}</span>
					<span className="ml-1 text-[16px] text-gray-400">{size.unit}</span>
				</div>
			</span>
		</div>
	);
};

// TODO: Replace string with Category enum type
const CategoryToIcon: Record<string, string> = {
	Recents: 'Collection',
	Favorites: 'HeartFlat',
	Photos: 'Image',
	Videos: 'Video',
	Music: 'Audio',
	Documents: 'Document',
	Downloads: 'Package',
	Applications: 'Application',
	Games: "Game",
	Books: 'Book',
	Encrypted: 'Encrypted',
	Archives: 'Database',
	Projects: 'Folder',
	Trash: 'Trash'
};

// Map the category to the ObjectKind for searching
const SearchableCategories: Record<string, ObjectKindKey> = {
	Photos: 'Image',
	Videos: 'Video',
	Music: 'Audio',
	Documents: 'Document',
	Encrypted: 'Encrypted'
}

export type SearchArgs = z.infer<typeof SEARCH_PARAMS>;

export const Component = () => {
	const platform = usePlatform();
	const explorerStore = useExplorerStore();
	const { library } = useLibraryContext();
	const stats = useLibraryQuery(['library.getStatistics'], {
		initialData: { ...EMPTY_STATISTICS }
	});
	const { explorerViewOptions, explorerControlOptions } = useExplorerTopBarOptions();

	const [selectedCategory, setSelectedCategory] = useState<string>('Recents');

	const canSearch = !!SearchableCategories[selectedCategory];

	const recentFiles = useLibraryQuery(['files.getRecent', 50]);

	const kind = [ObjectKind[SearchableCategories[selectedCategory] || 0] as number];

	const searchQuery = useLibraryQuery(['search.paths', { kind }], {
		suspense: true,
		enabled: canSearch
	});

	const searchItems = useMemo(() => {
		if (explorerStore.layoutMode !== 'media') return searchQuery.data?.items;

		return searchQuery.data?.items.filter((item) => {
			const { kind } = getExplorerItemData(item);
			return kind === 'Video' || kind === 'Image';
		});
	}, [searchQuery.data, explorerStore.layoutMode]);

	const categories = useLibraryQuery(['categories.list']);

	const haveRecentFiles = (recentFiles.data?.length || 0) > 0;

	overviewMounted = true;

	const inspector = explorerControlOptions.find(
		(t) => t.toolTipLabel === 'Show Inspector'
	) as ToolOption;
	const toolsViewOptions = haveRecentFiles
		? explorerViewOptions.filter(
			(o) =>
				o.toolTipLabel === 'Grid view' ||
				o.toolTipLabel === 'List view' ||
				o.toolTipLabel === 'Media view'
		)
		: [];

	let items: ExplorerItem[] = [];
	switch (selectedCategory) {
		case 'Recents':
			items = recentFiles.data || [];
			break;
		default:
			if (canSearch) {
				items = searchItems || [];
			}
	}

	const page = usePageLayout();

	return (
		<div>
			<TopBarChildren toolOptions={[toolsViewOptions, [inspector]]} />
			<Explorer
				inspectorClassName="!pt-0 !fixed !top-[50px] !right-[10px] !w-[260px]"
				viewClassName="!pl-0 !pt-0 !h-auto"
				explorerClassName="!overflow-visible"
				items={items}
				scrollRef={page?.ref}
			>
				{/* STAT HEADER */}
				<div className="flex w-full">
					{/* STAT CONTAINER */}
					<div className="-mb-1 flex h-20 overflow-hidden">
						{Object.entries(stats?.data || []).map(([key, value]) => {
							if (!displayableStatItems.includes(key)) return null;
							return (
								<StatItem
									key={`${library.uuid} ${key}`}
									title={StatItemNames[key as keyof Statistics]!}
									bytes={BigInt(value)}
									isLoading={platform.demoMode ? false : stats.isLoading}
								/>
							);
						})}
					</div>
				</div>
				<div className="sticky top-0 z-50 mt-4 flex flex-wrap space-x-[1px] bg-app py-1">
					{categories.data?.map((category) => {
						const iconString = CategoryToIcon[category] || 'Document';
						const icon = icons[iconString as keyof typeof icons];
						return (
							<CategoryButton
								key={category}
								category={category}
								icon={icon}
								items={0}
								selected={selectedCategory === category}
								onClick={() => setSelectedCategory(category)}
							/>
						);
					})}
				</div>
			</Explorer>
		</div>
	);
};

interface CategoryButtonProps {
	category: string;
	items: number;
	icon: string;
	selected?: boolean;
	onClick?: () => void;
}

function CategoryButton({ category, icon, items, selected, onClick }: CategoryButtonProps) {
	return (
		<div
			onClick={onClick}
			className={clsx(
				'flex shrink-0 items-center rounded-md px-1.5 py-1 text-sm',
				selected && 'bg-app-selected/20'
			)}
		>
			<img src={icon} className="mr-3 h-12 w-12" />
			<div className="pr-5">
				<h2 className="text-sm font-medium">{category}</h2>
				{items !== undefined && (
					<p className="text-xs text-ink-faint">
						{items} Item{items > 1 && 's'}
					</p>
				)}
			</div>
		</div>
	);
}
