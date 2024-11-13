import { NativeModulesProxy, EventEmitter, Subscription } from 'expo-modules-core';

// Import the native module. On web, it will be resolved to NativeFunctions.web.ts
// and on native platforms to NativeFunctions.ts
import NativeFunctionsModule from './NativeFunctionsModule';
import NativeFunctionsView from './NativeFunctionsView';
import { ChangeEventPayload, NativeFunctionsViewProps } from './NativeFunctions.types';

// Get the native constant value.
export const PI = NativeFunctionsModule.PI;

export function hello(): string {
  return NativeFunctionsModule.hello();
}

export async function setValueAsync(value: string) {
  return await NativeFunctionsModule.setValueAsync(value);
}

const emitter = new EventEmitter(NativeFunctionsModule ?? NativeModulesProxy.NativeFunctions);

export function addChangeListener(listener: (event: ChangeEventPayload) => void): Subscription {
  return emitter.addListener<ChangeEventPayload>('onChange', listener);
}

export { NativeFunctionsView, NativeFunctionsViewProps, ChangeEventPayload };
