import * as RadixCM from '@radix-ui/react-context-menu';
import { VariantProps, cva } from 'class-variance-authority';
import clsx from 'clsx';
import { CaretRight, Icon, IconProps } from 'phosphor-react';
import { ContextType, PropsWithChildren, Suspense, createContext, useContext } from 'react';

interface ContextMenuProps extends RadixCM.MenuContentProps {
	trigger: React.ReactNode;
	onOpenChange?: (open: boolean) => void;
	disabled?: boolean;
}

export const contextMenuClassNames = clsx(
	'z-50 max-h-[calc(100vh-20px)] overflow-y-auto',
	'my-2 min-w-[12rem] max-w-[16rem] py-0.5',
	'cool-shadow bg-menu',
	'border border-menu-line',
	'cursor-default select-none rounded-md',
	'animate-in fade-in'
);

const ContextMenuContext = createContext(false);

export const useContextMenuContext = <T extends boolean>({ suspense }: { suspense?: T } = {}) => {
	const ctx = useContext(ContextMenuContext);

	if (suspense && ctx === null) throw new Error('ContextMenuContext.Provider not found!');

	return ctx as T extends true
		? NonNullable<ContextType<typeof ContextMenuContext>>
		: NonNullable<ContextType<typeof ContextMenuContext>> | undefined;
};

const Root = ({
	trigger,
	children,
	className,
	onOpenChange,
	disabled,
	...props
}: ContextMenuProps) => {
	return (
		<RadixCM.Root onOpenChange={onOpenChange}>
			<RadixCM.Trigger asChild onContextMenu={(e) => disabled && e.preventDefault()}>
				{trigger}
			</RadixCM.Trigger>
			<RadixCM.Portal>
				<RadixCM.Content className={clsx(contextMenuClassNames, className)} {...props}>
					<ContextMenuContext.Provider value={true}>
						{children}
					</ContextMenuContext.Provider>
				</RadixCM.Content>
			</RadixCM.Portal>
		</RadixCM.Root>
	);
};

export const contextMenuSeparatorClassNames = 'border-b-menu-line mx-1 my-0.5 border-b';

const Separator = (props: { className?: string }) => (
	<RadixCM.Separator className={clsx(contextMenuSeparatorClassNames, props.className)} />
);

const SubMenu = ({
	label,
	icon,
	className,
	...props
}: RadixCM.MenuSubContentProps & ContextMenuItemProps) => {
	return (
		<RadixCM.Sub>
			<RadixCM.SubTrigger className={contextMenuItemClassNames}>
				<ContextMenuDivItem rightArrow {...{ label, icon }} />
			</RadixCM.SubTrigger>
			<RadixCM.Portal>
				<Suspense fallback={null}>
					<RadixCM.SubContent
						className={clsx(contextMenuClassNames, '-mt-2', className)}
						{...props}
					/>
				</Suspense>
			</RadixCM.Portal>
		</RadixCM.Sub>
	);
};

const contextMenuItemStyles = cva(
	[
		'flex h-[26px] items-center space-x-2 overflow-hidden rounded px-2',
		'text-sm text-menu-ink',
		'group-radix-highlighted:text-white',
		'group-radix-disabled:pointer-events-none group-radix-disabled:text-menu-ink/50',
		'group-radix-state-open:bg-accent group-radix-state-open:text-white'
	],
	{
		variants: {
			variant: {
				default: 'group-radix-highlighted:bg-accent',
				danger: [
					'text-red-600 dark:text-red-400',
					'group-radix-highlighted:text-white',
					'group-radix-highlighted:bg-red-500'
				]
			}
		},
		defaultVariants: {
			variant: 'default'
		}
	}
);

export interface ContextMenuItemProps extends VariantProps<typeof contextMenuItemStyles> {
	icon?: Icon;
	iconProps?: IconProps;
	rightArrow?: boolean;
	label?: string;
	keybind?: string;
}

export const contextMenuItemClassNames = 'group py-0.5 outline-none px-1';

const Item = ({
	icon,
	label,
	rightArrow,
	children,
	keybind,
	variant,
	iconProps,
	onClick,
	...props
}: ContextMenuItemProps & RadixCM.MenuItemProps) => {
	return (
		<RadixCM.Item
			className={contextMenuItemClassNames}
			onClick={(e) => !props.disabled && onClick?.(e)}
			{...props}
		>
			<ContextMenuDivItem
				{...{ icon, iconProps, label, rightArrow, keybind, variant, children }}
			/>
		</RadixCM.Item>
	);
};

export const ContextMenuDivItem = ({
	variant,
	children,
	className,
	...props
}: PropsWithChildren<ContextMenuItemProps & { className?: string }>) => (
	<div className={contextMenuItemStyles({ variant, className })}>
		{children || <ItemInternals {...props} />}
	</div>
);

const ItemInternals = ({ icon, label, rightArrow, keybind, iconProps }: ContextMenuItemProps) => {
	const ItemIcon = icon;

	return (
		<>
			{ItemIcon && <ItemIcon size={18} {...iconProps} />}
			{label && <span className="flex-1 truncate">{label}</span>}

			{keybind && (
				<span className="text-xs font-medium group-radix-highlighted:text-white">
					{keybind}
				</span>
			)}
			{rightArrow && (
				<CaretRight
					weight="fill"
					size={12}
					className="text-menu-faint group-radix-highlighted:text-white group-radix-state-open:text-white"
				/>
			)}
		</>
	);
};

export const ContextMenu = {
	Root,
	Item,
	Separator,
	SubMenu
};
