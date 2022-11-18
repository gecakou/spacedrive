import { useEffect } from 'react';
import { FieldValues, UseFormReturn } from 'react-hook-form';
import { useDebouncedCallback } from 'use-debounce';

export function useAutoForm<TFieldValues extends FieldValues = FieldValues, TContext = any>(
	form: UseFormReturn<TFieldValues, TContext>,
	callback: (data: any) => void
) {
	const debounced = useDebouncedCallback(callback, 500);

	// listen for any form changes
	form.watch(debounced);

	// persist unchanged data when the component is unmounted
	useEffect(() => () => debounced.flush(), [debounced]);
}
