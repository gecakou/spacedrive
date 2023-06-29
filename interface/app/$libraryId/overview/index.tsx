import { getIcon } from '@sd/assets/util';
import { useMemo, useState } from 'react';
import 'react-loading-skeleton/dist/skeleton.css';
import { Category } from '@sd/client';
import { useIsDark } from '../../../hooks';
import { ExplorerContext } from '../Explorer/Context';
import ContextMenu from '../Explorer/ContextMenu';
// import ContextMenu from '../Explorer/FilePath/ContextMenu';
import { Inspector } from '../Explorer/Inspector';
import { DefaultTopBarOptions } from '../Explorer/TopBarOptions';
import View from '../Explorer/View';
import { useExplorerStore } from '../Explorer/store';
import { usePageLayout } from '../PageLayout';
import { TopBarPortal } from '../TopBar/Portal';
import Statistics from '../overview/Statistics';
import { Categories, CategoryList } from './Categories';
import { IconForCategory, useItems } from './data';

const IconToDescription = {
	Recents: "See files you've recently opened or created",
	Favorites: 'See files you have marked as favorites',
	Albums: 'Organize your photos and videos into albums',
	Photos: 'View all photos in your library',
	Videos: 'View all videos in your library',
	Movies: 'View all movies in your library',
	Music: 'View all music in your library',
	Documents: 'View all documents in your library',
	Downloads: 'View all downloads in your library',
	Encrypted: 'View all encrypted files in your library',
	Projects: 'View all projects in your library',
	Applications: 'View all applications in your library',
	Archives: 'View all archives in your library',
	Databases: 'View all databases in your library',
	Games: 'View all games in your library',
	Books: 'View all books in your library',
	Contacts: 'View all contacts in your library',
	Trash: 'View all files in your trash'
};

export const Component = () => {
	const explorerStore = useExplorerStore();
	const page = usePageLayout();
	const isDark = useIsDark();

	const [selectedCategory, setSelectedCategory] = useState<Category>('Recents');

	const { items, query, loadMore } = useItems(selectedCategory);

	const [selectedItemId, setSelectedItemId] = useState<number>();

	const selectedItem = useMemo(
		() => (selectedItemId ? items?.find((item) => item.item.id === selectedItemId) : undefined),
		[selectedItemId, items]
	);

	return (
		<ExplorerContext.Provider value={{}}>
			<TopBarPortal right={<DefaultTopBarOptions />} />

			<Statistics />

			<Categories selected={selectedCategory} onSelectedChanged={setSelectedCategory} />

			<div className="flex flex-1">
				<View
					items={query.isLoading ? null : items || []}
					// TODO: Fix this type here.
					scrollRef={page?.ref as any}
					onLoadMore={loadMore}
					rowsBeforeLoadMore={5}
					selected={selectedItemId}
					onSelectedChange={setSelectedItemId}
					top={68}
					className={explorerStore.layoutMode === 'rows' ? 'min-w-0' : undefined}
					contextMenu={selectedItem ? <ContextMenu item={selectedItem} /> : null}
					emptyNotice={
						<div className="flex h-full flex-col items-center justify-center text-white">
							<img
								src={getIcon(
									IconForCategory[selectedCategory] || 'Document',
									isDark
								)}
								className="h-32 w-32"
							/>
							<h1 className="mt-4 text-lg font-bold">{selectedCategory}</h1>
							<p className="mt-1 text-sm text-ink-dull">
								{IconToDescription[selectedCategory]}
							</p>
						</div>
					}
				/>

				{explorerStore.showInspector && (
					<Inspector
						data={selectedItem}
						showThumbnail={explorerStore.layoutMode !== 'media'}
						className="custom-scroll inspector-scroll sticky top-[68px] h-auto w-[260px] shrink-0 self-start bg-app pb-4 pl-1.5 pr-1"
					/>
				)}
			</div>
		</ExplorerContext.Provider>
	);
};
