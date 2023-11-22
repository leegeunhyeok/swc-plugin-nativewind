import { transform } from '@swc/core';
import highlight from 'cli-highlight';

const inputCode =`
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
`;

;(async () => {
  const { code: outputCode } = await transform(inputCode, {
    isModule: true,
    filename: 'Demo.tsx',
    jsc: {
      transform: {
        react: {
          runtime: 'automatic',
          importSource: 'react-native-css-interop',
        }
      },
      target: 'es5',
      parser: {
        syntax: 'typescript',
        tsx: true,
      },
      experimental: {
        plugins: [['.', {}]],
      },
      externalHelpers: false,
    },
  });

  console.log(highlight(outputCode, { language: 'js' }));
})();
