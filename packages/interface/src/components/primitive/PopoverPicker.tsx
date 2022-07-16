import clsx from 'clsx';
import React, { useCallback, useRef, useState } from 'react';
import { HexColorPicker } from 'react-colorful';

import useClickOutside from '../../hooks/useClickOutside';

interface PopoverPickerProps {
	color: string;
	onChange: (color: string) => void;
	className?: string;
}

export const PopoverPicker = ({ color, onChange, className }: PopoverPickerProps) => {
	const popover = useRef<HTMLDivElement | null>(null);
	const [isOpen, toggle] = useState(false);

	const close = useCallback(() => toggle(false), []);
	useClickOutside(popover, close);

	return (
		<div className={clsx('relative flex items-center mt-3', className)}>
			<div
				className={clsx('w-5 h-5 rounded-full shadow ', isOpen && 'dark:border-gray-500')}
				style={{ backgroundColor: color }}
				onClick={() => toggle(true)}
			/>
			{/* <span className="inline ml-2 text-sm text-gray-200">Pick Color</span> */}

			{isOpen && (
				<div
					style={{ top: 'calc(100% + 7px)' }}
					className="absolute left-0 rounded-md shadow"
					ref={popover}
				>
					<HexColorPicker color={color} onChange={onChange} />
				</div>
			)}
		</div>
	);
};
