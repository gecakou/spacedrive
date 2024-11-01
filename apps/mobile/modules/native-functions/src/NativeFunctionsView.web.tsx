import * as React from 'react';

import { NativeFunctionsViewProps } from './NativeFunctions.types';

export default function NativeFunctionsView(props: NativeFunctionsViewProps) {
  return (
    <div>
      <span>{props.name}</span>
    </div>
  );
}
