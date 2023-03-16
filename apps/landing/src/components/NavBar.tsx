import { AppLogo } from '@sd/assets/images';
import { Academia, Discord, Github } from '@icons-pack/react-simple-icons';
import clsx from 'clsx';
import { Book, Chat, DotsThreeVertical, MapPin, User } from 'phosphor-react';
import { PropsWithChildren, useEffect, useState } from 'react';
import * as router from 'vite-plugin-ssr/client/router';
import { Button, Dropdown } from '@sd/ui';
import { positions } from '../pages/careers.page';
import { getWindow } from '../utils';

function NavLink(props: PropsWithChildren<{ link?: string }>) {
	return (
		<a
			href={props.link ?? '#'}
			target={props.link?.startsWith('http') ? '_blank' : undefined}
			className="cursor-pointer p-4 text-gray-300 no-underline transition hover:text-gray-50"
			rel="noreferrer"
		>
			{props.children}
		</a>
	);
}

function link(path: string) {
	const selected = getWindow()?.location.href.includes(path);

	return {
		selected,
		onClick: () => router.navigate(path),
		className: clsx(selected && 'bg-accent/20')
	};
}

function redirect(href: string) {
	return () => (window.location.href = href);
}

export default function NavBar() {
	const [isAtTop, setIsAtTop] = useState(true);
	const window = getWindow();

	function onScroll() {
		if ((getWindow()?.pageYOffset || 0) < 20) setIsAtTop(true);
		else if (isAtTop) setIsAtTop(false);
	}

	useEffect(() => {
		if (!window) return;
		setTimeout(onScroll, 0);
		getWindow()?.addEventListener('scroll', onScroll);
		return () => getWindow()?.removeEventListener('scroll', onScroll);
		// eslint-disable-next-line react-hooks/exhaustive-deps
	}, []);

	return (
		<div
			className={clsx(
				'fixed z-[55] h-16 w-full border-b px-2 transition ',
				isAtTop
					? 'border-transparent bg-transparent'
					: 'border-gray-550 bg-gray-700/80 backdrop-blur'
			)}
		>
			<div className="relative m-auto flex h-full max-w-[100rem] items-center p-5">
				<a href="/" className="absolute flex flex-row items-center">
					<img src={AppLogo} className="z-30 mr-3 h-8 w-8" />
					<h3 className="text-xl font-bold text-white">Spacedrive</h3>
				</a>

				<div className="m-auto hidden space-x-4 text-white lg:block ">
					<NavLink link="/roadmap">Roadmap</NavLink>
					<NavLink link="/team">Team</NavLink>
					<NavLink link="/blog">Blog</NavLink>
					<NavLink link="/docs/product/getting-started/introduction">Docs</NavLink>
					<div className="relative inline">
						<NavLink link="/careers">Careers</NavLink>
						{positions.length > 0 ? (
							<span className="bg-primary/80 absolute -top-1 -right-2 rounded-md px-[5px] text-xs">
								{` ${positions.length} `}
							</span>
						) : null}
					</div>
				</div>
				<div className="flex-1 lg:hidden" />
				<Dropdown.Root
					button={
						<Button className="ml-[140px] hover:!bg-transparent" size="icon">
							<DotsThreeVertical weight="bold" className="h-6 w-6 " />
						</Button>
					}
					className="top-2 right-4 block h-6 w-44 text-white lg:hidden"
					itemsClassName="!rounded-2xl shadow-2xl shadow-black p-2 !bg-gray-850 mt-2 !border-gray-500 text-[15px]"
				>
					<Dropdown.Section>
						<Dropdown.Item
							icon={Github}
							onClick={redirect('https://github.com/spacedriveapp/spacedrive')}
						>
							Repository
						</Dropdown.Item>
						<Dropdown.Item icon={Discord} onClick={redirect('https://discord.gg/gTaF2Z44f5')}>
							Join Discord
						</Dropdown.Item>
					</Dropdown.Section>
					<Dropdown.Section>
						<Dropdown.Item icon={MapPin} {...link('/roadmap')}>
							Roadmap
						</Dropdown.Item>
						<Dropdown.Item icon={Book} {...link('/docs/product/getting-started/introduction')}>
							Docs
						</Dropdown.Item>
						<Dropdown.Item icon={User} {...link('/team')}>
							Team
						</Dropdown.Item>
						<Dropdown.Item icon={Chat} {...link('/blog')}>
							Blog
						</Dropdown.Item>
						<Dropdown.Item icon={Academia} {...link('/careers')}>
							Careers
							{positions.length > 0 ? (
								<span className="bg-primary ml-2 rounded-md px-[5px] py-px text-xs">
									{positions.length}
								</span>
							) : null}
						</Dropdown.Item>
					</Dropdown.Section>
				</Dropdown.Root>

				<div className="absolute right-3 hidden flex-row space-x-5 lg:flex">
					<a href="https://discord.gg/gTaF2Z44f5" target="_blank" rel="noreferrer">
						<Discord className="text-white" />
					</a>
					<a href="https://github.com/spacedriveapp/spacedrive" target="_blank" rel="noreferrer">
						<Github className="text-white" />
					</a>
				</div>
			</div>
		</div>
	);
}
