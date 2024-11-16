import { NativeModule, requireNativeModule } from 'expo';

import { NativeModulesModuleEvents } from './NativeModules.types';

declare class NativeModulesModule extends NativeModule<NativeModulesModuleEvents> {
	PI: number;
	hello(): string;
	setValueAsync(value: string): Promise<void>;
}

// This call loads the native module object from the JSI.
export default requireNativeModule<NativeModulesModule>('NativeModules');
