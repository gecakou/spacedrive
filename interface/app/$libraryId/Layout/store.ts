import { proxy, useSnapshot } from 'valtio';
import { valtioPersist } from '@sd/client';

const state = proxy({
	sidebar: { size: 180, collapsed: false }
});

// TODO: Rename storage key
export const layoutStore = valtioPersist('sd-layout', state);

export const useLayoutStore = () => useSnapshot(layoutStore);
