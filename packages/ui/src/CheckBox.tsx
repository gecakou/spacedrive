import * as Checkbox from '@radix-ui/react-checkbox';
import { VariantProps, cva } from 'class-variance-authority';
import { Check } from 'phosphor-react';
import { ComponentProps, forwardRef } from 'react';

const styles = cva(
	[
		'form-check-input float-left mt-1 mr-2 h-4 w-4 appearance-none rounded-sm border border-gray-300 bg-white bg-contain bg-center bg-no-repeat align-top transition duration-200',
		'checked:border-blue-600 checked:bg-blue-600 focus:outline-none '
	],
	{ variants: {} }
);

export interface CheckBoxProps extends ComponentProps<'input'>, VariantProps<typeof styles> {}

export const CheckBox = forwardRef<HTMLInputElement, CheckBoxProps>(
	({ className, ...props }, ref) => (
		<input {...props} type="checkbox" ref={ref} className={styles({ className })} />
	)
);

export interface RadixCheckboxProps extends ComponentProps<typeof Checkbox.Root> {
	label?: string;
}

// TODO: Replace above with this, requires refactor of usage
export const RadixCheckbox = (props: RadixCheckboxProps) => (
	<div className="align-center flex">
		<Checkbox.Root
			className="flex h-[17px] w-[17px] shrink-0 rounded-md bg-app-button"
			id={props.name}
			{...props}
		>
			<Checkbox.Indicator className="flex h-[17px] w-[17px] items-center justify-center rounded-md bg-accent">
				<Check weight="bold" />
			</Checkbox.Indicator>
		</Checkbox.Root>
		{props.label && (
			<label className=" ml-2 font-medium" htmlFor={props.name}>
				{props.label}
			</label>
		)}
	</div>
);
