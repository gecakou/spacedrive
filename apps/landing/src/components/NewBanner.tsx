import Link from 'next/link';

export interface NewBannerProps {
	headline: string;
	href: string;
	link: string;
}

const NewBanner: React.FC<NewBannerProps> = (props) => {
	const { headline, href, link } = props;

	return (
		<aside
			onClick={() => (window.location.href = href)}
			className="fade-in-whats-new z-10 mb-5 flex w-10/12 cursor-pointer flex-row rounded-full border border-gray-550/50 bg-gray-800/50 px-5 py-1.5 text-xs transition hover:border-blue-200/50 hover:bg-gray-750 sm:w-auto sm:text-base"
		>
			<strong className="truncate font-semibold text-gray-350">{headline}</strong>
			<div role="separator" className="h-22 mx-4 w-[1px] bg-gray-500" />
			<Link
				href={href}
				className="font-regular shrink-0 bg-gradient-to-r from-primary-400 to-blue-600 bg-clip-text text-transparent decoration-primary-600"
			>
				{link} <span aria-hidden="true">&rarr;</span>
			</Link>
		</aside>
	);
};

export default NewBanner;
