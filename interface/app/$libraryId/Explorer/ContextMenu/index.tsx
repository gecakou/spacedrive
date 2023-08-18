import { Plus } from 'phosphor-react';
import { PropsWithChildren, useMemo } from 'react';
import { ExplorerItem } from '@sd/client';
import { ContextMenu } from '@sd/ui';
import { isNonEmpty } from '~/util';
import { useExplorerContext } from '../Context';
import { Conditional, ConditionalGroupProps } from './ConditionalItem';
import * as FilePathItems from './FilePath/Items';
import * as ObjectItems from './Object/Items';
import * as SharedItems from './SharedItems';
import { ContextMenuContextProvider } from './context';

export * as SharedItems from './SharedItems';
export * as FilePathItems from './FilePath/Items';
export * as ObjectItems from './Object/Items';

const Items = ({ children }: PropsWithChildren) => (
	<>
		<Conditional items={[FilePathItems.OpenOrDownload]} />
		<SharedItems.OpenQuickView />

		<SeparatedConditional items={[SharedItems.Details]} />

		<ContextMenu.Separator />
		<Conditional
			items={[
				SharedItems.RevealInNativeExplorer,
				SharedItems.Rename,
				FilePathItems.CutCopyItems,
				SharedItems.Deselect
			]}
		/>

		{children}

		<ContextMenu.Separator />
		<SharedItems.Share />

		<SeparatedConditional items={[ObjectItems.AssignTag]} />

		<Conditional
			items={[
				FilePathItems.CopyAsPath,
				FilePathItems.Crypto,
				FilePathItems.Compress,
				ObjectItems.ConvertObject,
				FilePathItems.ParentFolderActions,
				FilePathItems.SecureDelete
			]}
		>
			{(items) => (
				<ContextMenu.SubMenu label="More actions..." icon={Plus}>
					{items}
				</ContextMenu.SubMenu>
			)}
		</Conditional>

		<SeparatedConditional items={[FilePathItems.Delete]} />
	</>
);

export default (props: PropsWithChildren<{ items?: ExplorerItem[]; custom?: boolean }>) => {
	const explorer = useExplorerContext();

	const selectedItems = useMemo(
		() => props.items || [...explorer.selectedItems],
		[explorer.selectedItems, props.items]
	);

	if (!isNonEmpty(selectedItems)) return null;

	return (
		<ContextMenuContextProvider selectedItems={selectedItems}>
			{props.custom ? <>{props.children}</> : <Items>{props.children}</Items>}
		</ContextMenuContextProvider>
	);
};

/**
 * A `Conditional` that inserts a `<ContextMenu.Separator />` above its items.
 */
const SeparatedConditional = ({ items, children }: ConditionalGroupProps) => (
	<Conditional items={items}>
		{(c) => (
			<>
				<ContextMenu.Separator />
				{children ? children(c) : c}
			</>
		)}
	</Conditional>
);
