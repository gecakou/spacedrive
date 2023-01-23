import React, { PropsWithChildren } from 'react';
import { PageContextBuiltIn } from 'vite-plugin-ssr';
import '@sd/ui/style';
import { Footer } from './components/Footer';
import NavBar from './components/NavBar';
import { PageContextProvider } from './renderer/usePageContext';
import './style.scss';

export default function App({
	children,
	pageContext
}: PropsWithChildren<{
	pageContext: PageContextBuiltIn;
}>) {
	return (
		<React.StrictMode>
			<PageContextProvider pageContext={pageContext}>
				{/* <Button
						href="#content"
						className="fixed left-0 z-50 mt-3 ml-8 duration-200 -translate-y-16 cursor-pointer focus:translate-y-0"
						variant="gray"
					>
						Skip to content
					</Button> */}

				<>
					<NavBar />
					<div className="dark dark:bg-black dark:text-white z-10 max-w-[100rem] m-auto">
						{children}
					</div>
					<Footer />
				</>
			</PageContextProvider>
		</React.StrictMode>
	);
}
