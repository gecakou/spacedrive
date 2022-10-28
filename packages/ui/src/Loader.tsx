import clsx from 'clsx';
import { Puff } from 'react-loading-icons';

export function Loader(props: { className?: string }) {
	return (
		<Puff
			stroke="#2599FF"
			strokeOpacity={4}
			strokeWidth={5}
			speed={1}
			className={clsx('w-7 h-7', props.className)}
		/>
	);
}
