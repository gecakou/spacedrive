import { useEffect } from 'react';
import { useLocation, useNavigate } from 'react-router';

import { KeybindEvent } from '../util/keybind';
import { useQuickRescan } from './useQuickRescan';
import { getWindowState } from './useWindowState';

export const useKeybindEventHandler = (libraryId?: string) => {
	const navigate = useNavigate();
	const windowState = getWindowState();
	const rescan = useQuickRescan();
	const regex = new RegExp(
		'/[0-9A-Fa-f]{8}-[0-9A-Fa-f]{4}-[0-9A-Fa-f]{4}-[0-9A-Fa-f]{4}-[0-9A-Fa-f]{12}/location/'
	);
	const id = parseInt(useLocation().pathname.replace(regex, ''));

	useEffect(() => {
		const handler = (e: KeybindEvent) => {
			e.preventDefault();

			switch (e.detail.action) {
				case 'open_settings':
					libraryId && navigate(`/${libraryId}/settings/client/general`);
					break;
				case 'reload_explorer':
					!isNaN(id) && rescan(id);
					break;
				// case 'open_overview':
				// 	libraryId && navigate(`/${libraryId}/overview`);
				// 	break;
				case 'open_search':
					// somehow emit ctrl/cmd+f
					break;
				case 'window_fullscreened':
					windowState.isFullScreen = true;
					break;
				case 'window_not_fullscreened':
					windowState.isFullScreen = false;
					break;
			}
		};

		document.addEventListener('keybindexec', handler);
		return () => document.removeEventListener('keybindexec', handler);
	}, [navigate, libraryId, windowState, rescan, id]);
};
