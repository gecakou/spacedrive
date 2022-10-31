import { VariantProps, cva, cx } from 'class-variance-authority';
import { forwardRef } from 'react';
import { Link, LinkProps } from 'react-router-dom';

export interface ButtonBaseProps extends VariantProps<typeof styles> {}

export type ButtonProps = ButtonBaseProps &
	React.ButtonHTMLAttributes<HTMLButtonElement> & {
		href?: undefined;
	};

export type LinkButtonProps = ButtonBaseProps &
	React.AnchorHTMLAttributes<HTMLAnchorElement> & {
		href?: string;
	};

type Button = {
	(props: ButtonProps): JSX.Element;
	(props: LinkButtonProps): JSX.Element;
};

const hasHref = (props: ButtonProps | LinkButtonProps): props is LinkButtonProps => 'href' in props;

const styles = cva(
	'border rounded-md items-center transition-colors duration-100 cursor-default disabled:opacity-50',
	{
		variants: {
			pressEffect: {
				true: 'active:translate-y-[1px]'
			},
			disabled: {
				true: 'opacity-70 pointer-events-none cursor-not-allowed'
			},
			size: {
				icon: '!p-1',
				md: 'py-1 px-3 text-md font-medium',
				sm: 'py-1 px-2 text-sm font-medium'
			},
			variant: {
				default: [
					'bg-app-button bg-transparent active:bg-app-selected hover:bg-app-hover',
					'border-transparent hover:border-app-line active:border-app-line'
				],
				outline: [
					'border-transparent hover:border-app-line/50 active:border-app-line active:bg-app-box/30'
				],
				gray: [
					'bg-app-button active:bg-app-selected hover:bg-app-hover',
					'border-app-line hover:border-app-line active:border-app-active'
				],
				accent: [
					'bg-accent text-white active:bg-accent hover:bg-accent-faint border-accent-deep hover:border-accent active:border-accent-deep shadow-md shadow-app-shade/10'
				],
				colored: ['text-white shadow-sm hover:bg-opacity-90 active:bg-opacity-100'],
				bare: ''
			}
		},
		defaultVariants: {
			size: 'md',
			variant: 'default'
		}
	}
);

export const Button = forwardRef<
	HTMLButtonElement | HTMLAnchorElement,
	ButtonProps | LinkButtonProps
>(({ className, ...props }, ref) => {
	className = cx(styles(props), className);
	return hasHref(props) ? (
		<a {...props} ref={ref as any} className={cx(className, 'no-underline inline-block')} />
	) : (
		<button {...(props as ButtonProps)} ref={ref as any} className={className} />
	);
});

export const ButtonLink = forwardRef<
	HTMLLinkElement,
	ButtonBaseProps & LinkProps & React.RefAttributes<HTMLAnchorElement>
>(({ className, to, ...props }, ref) => {
	className = cx(
		styles(props),
		'no-underline disabled:opacity-50 disabled:cursor-not-allowed',
		className
	);

	return (
		<Link to={to} ref={ref as any} className={className}>
			{props.children}
		</Link>
	);
});
