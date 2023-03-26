import { VariantProps, cva, cx } from 'class-variance-authority';
import clsx from 'clsx';
import { Eye, EyeSlash, MagnifyingGlass } from 'phosphor-react';
import { PropsWithChildren, forwardRef, useState } from 'react';
import { Button } from './Button';

export interface InputBaseProps extends VariantProps<typeof inputStyles> {
	label?: string;
	error?: string | boolean;
	icon?: React.ReactNode;
	iconPosition?: 'left' | 'right';
	right?: React.ReactNode;
	outerClassName?: string;
	labelClassName?: string;
	iconClassName?: string;
	rightClassName?: string;
}

export type InputProps = InputBaseProps & Omit<React.ComponentProps<'input'>, 'size'>;

export type TextareaProps = InputBaseProps & React.ComponentProps<'textarea'>;

export const inputStyles = cva(
	[
		'w-full',
		'rounded-md border text-sm leading-7',
		'shadow-sm outline-none transition-all focus-within:ring-2'
	],
	{
		variants: {
			variant: {
				default: [
					'bg-app-input focus-within:bg-app-focus placeholder-ink-faint border-app-line',
					'focus-within:ring-app-selected/30 focus-within:border-app-divider/80'
				]
			},
			size: {
				sm: 'h-[30px]',
				md: 'h-[34px]',
				lg: 'h-[38px]'
			}
		},
		defaultVariants: {
			variant: 'default',
			size: 'sm'
		}
	}
);

export const Input = forwardRef<HTMLInputElement, InputProps>(
	(
		{
			variant,
			size,
			label,
			error,
			icon,
			right,
			className,
			outerClassName,
			labelClassName,
			iconClassName,
			rightClassName,
			iconPosition = 'left',
			required,
			...props
		},
		ref
	) => (
		<div className={outerClassName}>
			{label && (
				<label htmlFor={props.id} className={clsx('mb-1 flex text-sm font-medium', labelClassName)}>
					{label}
					{required && <span className="ml-1 text-red-500">*</span>}
				</label>
			)}

			<div
				className={clsx(
					'group flex',
					error && 'border-red-500 focus-within:border-red-500 focus-within:ring-red-400/30',
					inputStyles({ variant, size: right && !size ? 'md' : size, className })
				)}
			>
				<div
					className={clsx(
						'flex h-full flex-1 overflow-hidden',
						iconPosition === 'right' && 'flex-row-reverse'
					)}
				>
					{icon && (
						<div
							className={clsx(
								'flex h-full items-center',
								iconPosition === 'left' ? 'pr-2 pl-[10px]' : 'pl-2 pr-[10px]',
								iconClassName
							)}
						>
							{icon}
						</div>
					)}

					<input
						className={clsx(
							'placeholder:text-ink-faint flex-1 truncate border-none bg-transparent px-3 text-sm outline-none',
							(right || (icon && iconPosition === 'right')) && 'pr-0',
							icon && iconPosition === 'left' && 'pl-0'
						)}
						ref={ref}
						{...props}
					/>
				</div>

				{right && (
					<div className={clsx('flex h-full items-center px-1', rightClassName)}>{right}</div>
				)}
			</div>

			{error && typeof error === 'string' && (
				<span className="whitespace-pre-line text-xs text-red-500">{error}</span>
			)}
		</div>
	)
);

export const SearchInput = forwardRef<HTMLInputElement, InputProps>((props, ref) => (
	<Input {...props} ref={ref} icon={<MagnifyingGlass className="text-gray-350" size={18} />} />
));

export const TextArea = ({ size, variant, ...props }: TextareaProps) => {
	return (
		<textarea
			{...props}
			className={clsx('h-auto px-3 py-2', inputStyles({ size, variant }), props.className)}
		/>
	);
};

export function Label(props: PropsWithChildren<{ slug?: string }>) {
	return (
		<label className="text-sm font-bold" htmlFor={props.slug}>
			{props.children}
		</label>
	);
}

interface PasswordInputProps extends InputProps {
	buttonClassnames?: string;
}

export const PasswordInput = forwardRef<HTMLInputElement, PasswordInputProps>((props, ref) => {
	const [showPassword, setShowPassword] = useState(false);

	const CurrentEyeIcon = showPassword ? EyeSlash : Eye;

	return (
		<Input
			{...props}
			type={showPassword ? 'text' : 'password'}
			ref={ref}
			right={
				<Button
					onClick={() => setShowPassword(!showPassword)}
					size="icon"
					className={clsx(props.buttonClassnames)}
				>
					<CurrentEyeIcon className="h-4 w-4" />
				</Button>
			}
		/>
	);
});
