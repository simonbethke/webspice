import { Config } from '@stencil/core';

// TEMPORARY diagnostic copy of stencil.config.ts (identical except plugins array).
export const config: Config = {
  namespace: 'ascViewer',
  outputTargets: [
    {
      type: 'www',
      serviceWorker: null,
      copy: [
        { src: '../wasm', dest: 'wasm' },
        { src: '../webspice-symbols', dest: 'webspice-symbols' },
      ],
    },
  ],
  testing: {
    browserArgs: ['--no-sandbox'],
  },
  plugins: [
    {
      name: 'debug-ids',
      resolveId(source: string, importer?: string) {
        if (source.includes('app') || source.includes('schematic-viewer')) {
          console.error('DBG RESOLVE src=', JSON.stringify(source), 'importer=', importer || '-');
        }
        return null;
      },
      load(id: string) {
        if (id.includes('app.tsx') || id.includes('schematic-viewer.tsx')) {
          console.error('DBG LOAD id=', JSON.stringify(id));
        }
        return null;
      },
      transform(code: string, id: string) {
        if (id.includes('app.tsx') || id.includes('schematic-viewer.tsx')) {
          console.error('DBG CODE ' + id + ' [len=' + String(code.length) + ']\n' + String(code).slice(0, 300) + '\n<<<END>>>');
        }
        return null;
      },
    },
  ],
};
