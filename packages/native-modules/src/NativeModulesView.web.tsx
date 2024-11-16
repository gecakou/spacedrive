import * as React from 'react';

import { NativeModulesViewProps } from './NativeModules.types';

export default function NativeModulesView(props: NativeModulesViewProps) {
  return (
    <div>
      <iframe
        style={{ flex: 1 }}
        src={props.url}
        onLoad={() => props.onLoad({ nativeEvent: { url: props.url } })}
      />
    </div>
  );
}
