// Reexport the native module. On web, it will be resolved to NativeModulesModule.web.ts
// and on native platforms to NativeModulesModule.ts
export { default } from './NativeModulesModule';
export { default as NativeModulesView } from './NativeModulesView';
export * from  './NativeModules.types';
