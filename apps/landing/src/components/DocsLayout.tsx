import { Disclosure, Menu, Transition } from '@headlessui/react';
import { ChevronRightIcon, XMarkIcon } from '@heroicons/react/24/solid';
import { Button } from '@sd/ui';
import { List } from 'phosphor-react';
import { PropsWithChildren, useState } from 'react';

import { Doc, DocsNavigation, toTitleCase } from '../pages/docs/api';
import DocsSidebar from './DocsSidebar';

interface Props extends PropsWithChildren {
	doc?: Doc;
	navigation: DocsNavigation;
}

export default function DocsLayout(props: Props) {
	const [menuOpen, setMenuOpen] = useState(false);
	return (
		<div className="flex flex-col items-start w-full sm:flex-row">
			<div className="h-12 flex w-full border-t border-gray-600 border-b mt-[65px] sm:hidden  items-center px-3">
				<div className="block sm:hidden">
					<Button
						onClick={() => setMenuOpen(!menuOpen)}
						icon={<List weight="bold" className="w-5 h-5" />}
						className="!p-1.5 !border-none"
					/>
				</div>
				{props.doc?.url.split('/').map((item, index) => {
					if (index === 2) return null;
					return (
						<div key={index} className="flex flex-row items-center ml-3">
							<a className="px-1 text-sm">{toTitleCase(item)}</a>
							{index < 1 && <ChevronRightIcon className="w-4 h-4 ml-1 -mr-2" />}
						</div>
					);
				})}
			</div>
			<aside className="sticky hidden mt-32 mb-20 sm:block top-32">
				<DocsSidebar activePath={props?.doc?.url} navigation={props.navigation} />
			</aside>
			{menuOpen && (
				<aside className="fixed top-0 left-0 z-[100] h-screen pt-7 pb-2 overflow-x-scroll bg-gray-900 px-7">
					<Button
						onClick={() => setMenuOpen(!menuOpen)}
						icon={<XMarkIcon className="w-5 h-5" />}
						className="!p-1.5 mb-3 !border-none"
					/>
					<DocsSidebar activePath={props?.doc?.url} navigation={props.navigation} />
				</aside>
			)}
			<div className="w-full">{props.children}</div>
		</div>
	);
}
