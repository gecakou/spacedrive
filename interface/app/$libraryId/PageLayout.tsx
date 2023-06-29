import { RefObject, createContext, useContext, useRef } from 'react';
import { Outlet } from 'react-router';
import { TOP_BAR_HEIGHT } from './TopBar';

const PageContext = createContext<{ ref: RefObject<HTMLDivElement> } | undefined>(undefined);
export const usePageLayout = () => useContext(PageContext);

export const Component = () => {
	const ref = useRef<HTMLDivElement>(null);

	return (
		<div
			ref={ref}
			className="custom-scroll topbar-page-scroll app-background  flex w-full flex-1 flex-col"
			style={{ paddingTop: TOP_BAR_HEIGHT }}
		>
			<PageContext.Provider value={{ ref }}>
				<div className="flex w-full flex-1 flex-col">
					<Outlet />
				</div>
			</PageContext.Provider>
		</div>
	);
};
