# swc-plugin-nativewind

Swc plugin implementation of the [NativeWind](https://github.com/marklawlor/nativewind) babel plugin.

✨ `100%` coverage of [test cases](https://github.com/marklawlor/nativewind/blob/main/packages/react-native-css-interop/src/__tests__/babel-plugin.ts) in NativeWind. (checkout the test cases [here](https://github.com/leegeunhyeok/swc-plugin-nativewind/tree/master/transform/tests/fixture))

## Installation

```bash
npm install swc-plugin-nativewind
# or yarn
yarn add swc-plugin-nativewind
```

## Usage

```ts
import { transform } from '@swc/core';

await transform(code, {
  jsc: {
    transform: {
      react: {
        runtime: 'automatic',
        importSource: 'react-native-css-interop',
      },
    },
    experimental: {
      plugins: [['swc-plugin-nativewind', {}]],
    },
    externalHelpers: false,
  },
});
```

## Preview

Before

```tsx
import React, { createElement } from 'react';
import { Container, Section } from '@app/components';

export function Demo(): JSX.Element {
  return (
    <Container>
      <Section>{React.createElement('h1', null)}</Section>
      <Section>{createElement('div', null)}</Section>
    </Container>
  );
};
```

After

```js
import { jsx as _jsx, jsxs as _jsxs } from "react-native-css-interop/jsx-runtime";
import { createElementAndCheckCssInterop as __c } from "react-native-css-interop";
import React, { createElement } from "react";
import { Container, Section } from "@app/components";
export function MyComponent() {
  return /*#__PURE__*/ _jsxs(Container, {
    children: [
      /*#__PURE__*/ _jsx(Section, {
        children: __c("h1", null)
      }),
      /*#__PURE__*/ _jsx(Section, {
        children: __c("div", null)
      })
    ]
  });
}
```
