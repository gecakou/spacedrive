import clsx from 'clsx';
import { memo } from 'react';
import { ExplorerItem, byteSize, getItemFilePath, getItemLocation } from '@sd/client';
import { ViewItem } from '.';
import FileThumb from '../FilePath/Thumb';
import { useExplorerViewContext } from '../ViewContext';
import { useExplorerStore } from '../store';
import GridList from './GridList';
import RenamableItemText from './RenamableItemText';

interface GridViewItemProps {
	data: ExplorerItem;
	selected: boolean;
	isRenaming: boolean;
	cut: boolean;
}

const GridViewItem = memo(({ data, selected, cut, isRenaming }: GridViewItemProps) => {
	const filePathData = getItemFilePath(data);
	const location = getItemLocation(data);
	const { showBytesInGridView, gridItemSize } = useExplorerStore();

	const showSize =
		!filePathData?.is_dir &&
		!location &&
		showBytesInGridView &&
		(!isRenaming || (isRenaming && !selected));

	return (
		<ViewItem data={data} className="h-full w-full">
			<div className={clsx('mb-1 rounded-lg ', selected && 'bg-app-selectedItem')}>
				<FileThumb
					data={data}
					size={gridItemSize}
					className={clsx('mx-auto', cut && 'opacity-60')}
				/>
			</div>

			<div className="flex flex-col justify-center">
				<RenamableItemText
					item={data}
					selected={selected}
					style={{ maxHeight: gridItemSize / 3 }}
				/>
				{showSize && filePathData?.size_in_bytes_bytes && (
					<span
						className={clsx(
							'cursor-default truncate rounded-md px-1.5 py-[1px] text-center text-tiny text-ink-dull '
						)}
					>
						{`${byteSize(filePathData.size_in_bytes_bytes)}`}
					</span>
				)}
			</div>
		</ViewItem>
	);
});

export default () => {
	const explorerView = useExplorerViewContext();

	return (
		<GridList>
			{({ item, selected, cut }) => (
				<GridViewItem
					data={item}
					selected={selected}
					cut={cut}
					isRenaming={explorerView.isRenaming}
				/>
			)}
		</GridList>
	);
};
