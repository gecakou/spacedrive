import { requireNativeViewManager } from 'expo-modules-core';
import * as React from 'react';

import { NativeFunctionsViewProps } from './NativeFunctions.types';

const NativeView: React.ComponentType<NativeFunctionsViewProps> =
  requireNativeViewManager('NativeFunctions');

export default function NativeFunctionsView(props: NativeFunctionsViewProps) {
  return <NativeView {...props} />;
}
