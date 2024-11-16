import { requireNativeView } from 'expo';
import * as React from 'react';

import { NativeModulesViewProps } from './NativeModules.types';

const NativeView: React.ComponentType<NativeModulesViewProps> =
  requireNativeView('NativeModules');

export default function NativeModulesView(props: NativeModulesViewProps) {
  return <NativeView {...props} />;
}
