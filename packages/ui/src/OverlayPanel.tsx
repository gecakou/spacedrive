import * as DropdownMenu from '@radix-ui/react-dropdown-menu';
import clsx from 'clsx';
import { PropsWithChildren, useState } from 'react';
import { animated, config, useTransition } from 'react-spring';

interface Props extends DropdownMenu.MenuContentProps {
	trigger: React.ReactNode;
	transformOrigin?: string;
	disabled?: boolean;
}

export const OverlayPanel = ({
	trigger,
	children,
	disabled,
	transformOrigin,
	className,
	...props
}: PropsWithChildren<Props>) => {
	const [open, setOpen] = useState(false);

	// const transitions = useTransition(open, {
	// 	from: {
	// 		opacity: 0,
	// 		transform: `scale(0.5)`,
	// 		transformOrigin: transformOrigin || 'top'
	// 	},
	// 	enter: { opacity: 1, transform: 'scale(1)' },
	// 	leave: { opacity: -0.5, transform: 'scale(0.95)' },
	// 	config: { mass: 0.4, tension: 170, friction: 10 }
	// });

	return (
		<DropdownMenu.Root open={open} onOpenChange={setOpen}>
			<DropdownMenu.Trigger disabled={disabled} asChild>
				{trigger}
			</DropdownMenu.Trigger>
			{open && (
				<DropdownMenu.Portal forceMount>
					<DropdownMenu.Content forceMount asChild>
						<div
							className={clsx(
								'flex flex-col',
								'min-w-[11rem] z-50 m-2 space-y-1',
								'select-none cursor-default rounded-lg',
								'text-left text-sm text-ink',
								'bg-app-overlay ',
								'border border-app-line',
								'shadow-2xl shadow-black/60 ',
								className
							)}
							// style={styles}
						>
							{children}
						</div>
					</DropdownMenu.Content>
				</DropdownMenu.Portal>
			)}
		</DropdownMenu.Root>
	);
};
