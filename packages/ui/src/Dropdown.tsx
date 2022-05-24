import { Menu } from '@headlessui/react';
import { ChevronDownIcon } from '@heroicons/react/solid';
import clsx from 'clsx';
import React from 'react';

import { Button } from './Button';

type Section = {
	name: string;
	icon?: any;
	selected?: boolean;
	onPress?: () => any;
}[];

export interface DropdownProps {
	items: Section[];
	buttonText?: string;
	buttonProps: React.ComponentProps<typeof Button>;
	buttonIcon?: any;
	className?: string;
	itemsClassName?: string;
}

export const Dropdown: React.FC<DropdownProps> = (props) => {
	return (
		<div className={clsx('w-full mt-2', props.className)}>
			<Menu as="div" className="relative flex w-full text-left">
				<Menu.Button as="div" className="flex-1 outline-none">
					<Button size="sm" {...props.buttonProps}>
						{props.buttonIcon}
						{props.buttonText && (
							<>
								<span className="w-32 truncate"> {props.buttonText}</span>
								<div className="flex-grow" />
								<ChevronDownIcon
									className="w-5 h-5 ml-2 -mr-1 text-violet-200 hover:text-violet-100 "
									aria-hidden="true"
								/>
							</>
						)}
					</Button>
				</Menu.Button>

				<Menu.Items
					className={clsx(
						'absolute z-50 w-full bg-white border divide-y divide-gray-100 rounded shadow-xl top-full dark:bg-gray-550 dark:divide-gray-500 dark:border-gray-600 ring-1 ring-black ring-opacity-5 focus:outline-none',
						props.itemsClassName
					)}
				>
					{props.items.map((item, index) => (
						<div key={index} className="px-1 py-1">
							{item.map((button, index) => (
								<Menu.Item key={index}>
									{({ active }) => (
										<button
											onClick={button.onPress}
											className={clsx(
												'text-sm group flex rounded items-center w-full px-2 py-1 mb-[2px] dark:hover:bg-gray-500',
												{
													'bg-gray-300 dark:!bg-gray-500 dark:hover:bg-gray-500': button.selected
													// 'text-gray-900 dark:text-gray-200': !active
												}
											)}
										>
											{button.icon && (
												<button.icon
													className={clsx('mr-2 w-4 h-4', {
														'dark:text-gray-100': active,
														'text-gray-600 dark:text-gray-200': !active
													})}
												/>
											)}
											<span className="text-left">{button.name}</span>
										</button>
									)}
								</Menu.Item>
							))}
						</div>
					))}
				</Menu.Items>
			</Menu>
		</div>
	);
};
