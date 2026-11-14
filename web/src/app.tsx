import { Component, h } from '@stencil/core';

/**
 * App-Wurzel-Element: trägt den Schematic-Viewer.
 */
@Component({
  tag: 'app-root',
})
export class AppRoot {
  render() {
    return <schematic-viewer></schematic-viewer>;
  }
}
