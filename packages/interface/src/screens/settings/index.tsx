import { RouteObject } from 'react-router-dom';
import { lazyEl } from '~/util';

const screens: RouteObject[] = [
	{ path: 'general', element: lazyEl(() => import('./client/GeneralSettings')) },
	{ path: 'appearance', element: lazyEl(() => import('./client/AppearanceSettings')) },
	{ path: 'keybindings', element: lazyEl(() => import('./client/KeybindingSettings')) },
	{ path: 'extensions', element: lazyEl(() => import('./client/ExtensionsSettings')) },
	{ path: 'p2p', element: lazyEl(() => import('./node/P2PSettings')) },
	{ path: 'contacts', element: lazyEl(() => import('./library/ContactsSettings')) },
	{ path: 'experimental', element: lazyEl(() => import('./node/ExperimentalSettings')) },
	{ path: 'keys', element: lazyEl(() => import('./library/KeysSetting')) },
	{ path: 'libraries', element: lazyEl(() => import('./node/LibrariesSettings')) },
	{ path: 'security', element: lazyEl(() => import('./library/SecuritySettings')) },
	{ path: 'locations', element: lazyEl(() => import('./library/LocationsSettings')) },
	{ path: 'sharing', element: lazyEl(() => import('./library/SharingSettings')) },
	{ path: 'sync', element: lazyEl(() => import('./library/SyncSettings')) },
	{ path: 'tags', element: lazyEl(() => import('./library/TagsSettings')) },
	{ path: 'library', element: lazyEl(() => import('./library/LibraryGeneralSettings')) },
	{ path: 'tags', element: lazyEl(() => import('./library/TagsSettings')) },
	{ path: 'nodes', element: lazyEl(() => import('./library/NodesSettings')) },
	{ path: 'privacy', element: lazyEl(() => import('./client/PrivacySettings')) },
	{ path: 'about', element: lazyEl(() => import('./info/AboutSpacedrive')) },
	{ path: 'changelog', element: lazyEl(() => import('./info/Changelog')) },
	{ path: 'dependencies', element: lazyEl(() => import('./info/Dependencies')) },
	{ path: 'support', element: lazyEl(() => import('./info/Support')) },
	{
		path: 'locations',
		element: lazyEl(() => import('./SettingsSubPage')),
		children: [
			{ index: true, element: lazyEl(() => import('./library/LocationsSettings')) },
			{ path: ':id', element: lazyEl(() => import('./library/location/EditLocation')) }
		]
	}
];

export default screens;
