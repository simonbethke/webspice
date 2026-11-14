import { Config } from '@stencil/core';

export const config: Config = {
  namespace: 'ascViewer',
  outputTargets: [
    {
      type: 'www',
      serviceWorker: null,
      // Committed WASM-Paket web/wasm/ nach www/wasm/ kopieren
      // (wird zur Laufzeit über document.baseURI geladen).
      // `src` wird relativ zu web/src/, `dest` relativ zu web/www/ aufgelöst.
      copy: [
        { src: '../wasm', dest: 'wasm' },
        // Statische Kurzbibliothek web/webspice-symbols/ (committed, .asy)
        // — wird vom Viewer beim Start via fetch geladen (docs „Web-Frontend“).
        { src: '../webspice-symbols', dest: 'webspice-symbols' },
      ],
    },
  ],
  testing: {
    browserArgs: ['--no-sandbox'],
  },
};
