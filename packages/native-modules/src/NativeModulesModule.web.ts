import { registerWebModule, NativeModule } from 'expo';

import { NativeModulesModuleEvents } from './NativeModules.types';

class NativeModulesModule extends NativeModule<NativeModulesModuleEvents> {
  PI = Math.PI;
  async setValueAsync(value: string): Promise<void> {
    this.emit('onChange', { value });
  }
  hello() {
    return 'Hello world! 👋';
  }
}

export default registerWebModule(NativeModulesModule);
