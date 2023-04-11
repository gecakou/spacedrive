import clsx from 'clsx';
import { AnimatePresence, motion } from 'framer-motion';
import { CaretLeft, CaretRight, MagnifyingGlass } from 'phosphor-react';
import { useRef } from 'react';
import { useState } from 'react';
import { useLocation, useNavigate } from 'react-router-dom';
import { Popover, Tooltip } from '@sd/ui';
import SearchBar from './SearchBar';
import TopBarButton from './TopBarButton';
import { RoutePaths, groupKeys, useToolBarRouteOptions } from './useToolBarOptions';

export default () => {
	const TOP_BAR_ICON_STYLE = 'm-0.5 w-5 h-5 text-ink-dull';
	const navigate = useNavigate();
	const [toggleSearch, setToggleSearch] = useState(false);

	//create function to focus on search box when cmd+k is pressed
	const searchRef = useRef<HTMLInputElement>(null);
	const { pathname } = useLocation();
	const getPageName = pathname.split('/')[2] as RoutePaths;
	const { toolBarRouteOptions } = useToolBarRouteOptions();
	const toggleSearchIconHandler = () => {
		setToggleSearch(!toggleSearch);
	};
	const lengthOfTools = toolBarRouteOptions[getPageName].options.reduce((acc, curr) => {
		return [...Object.values(curr)].reduce((acc, curr) => {
			return acc + curr.length;
		}, acc);
	}, 0);

	return (
		<div
			data-tauri-drag-region
			className={clsx(
				'duration-250 absolute top-0 z-20 flex grid h-[46px] w-full shrink-0 grid-cols-3 items-center justify-center overflow-hidden border-b border-sidebar-divider bg-app px-5 transition-[background-color] transition-[border-color] ease-out'
			)}
		>
			<div className="flex ">
				<Tooltip label="Navigate back">
					<TopBarButton onClick={() => navigate(-1)}>
						<CaretLeft weight="bold" className={TOP_BAR_ICON_STYLE} />
					</TopBarButton>
				</Tooltip>
				<Tooltip label="Navigate forward">
					<TopBarButton onClick={() => navigate(1)}>
						<CaretRight weight="bold" className={TOP_BAR_ICON_STYLE} />
					</TopBarButton>
				</Tooltip>
			</div>

			<div data-tauri-drag-region className="flex w-full flex-row justify-center">
				<div className="flex gap-2">
					{toolBarRouteOptions[getPageName].options.map((group) => {
						return (Object.keys(group) as groupKeys[]).map((groupKey) => {
							return group[groupKey]?.map(
								({ icon, onClick, popOverComponent, toolTipLabel, topBarActive }, index) => {
									return (
										<div key={toolTipLabel} className="flex items-center">
											<Tooltip label={toolTipLabel}>
												{popOverComponent ? (
													<Popover
														className="focus:outline-none"
														trigger={
															<TopBarButton active={topBarActive} onClick={onClick}>
																{icon}
															</TopBarButton>
														}
													>
														<div className="block w-[250px] ">{popOverComponent}</div>
													</Popover>
												) : (
													<TopBarButton active={topBarActive} onClick={onClick ?? undefined}>
														{icon}
													</TopBarButton>
												)}
											</Tooltip>
											{index === (group[groupKey]?.length as number) - 1 && (
												<div className="ml-2 h-[15px] w-0 border-l border-zinc-600" />
											)}
										</div>
									);
								}
							);
						});
					})}
					{!toggleSearch && (
						<Tooltip label="Search" className={`${lengthOfTools > 8 ? '' : 'sm:hidden'}`}>
							<TopBarButton onClick={toggleSearchIconHandler}>
								<MagnifyingGlass className="m-0.5 h-5 w-5 text-ink-dull" />
							</TopBarButton>
						</Tooltip>
					)}
				</div>
				<SearchBar
					setToggleSearch={(arg: boolean) => setToggleSearch(arg)}
					toggleSearch={toggleSearch}
					className={`ml-4 ${toggleSearch ? '' : 'hidden'} ${lengthOfTools > 8 ? '' : 'sm:flex'}`}
					ref={searchRef}
				/>
			</div>
		</div>
	);
};
