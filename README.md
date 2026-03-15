<img src="cb.png" alt="Citybound" width="192"/>

Citybound is a city building game with a focus on realism, collaborative planning and simulation of microscopic details. It is independently developed, open source and funded through Patreon.

* [Homepage](http://cityboundsim.com) (with screenshots and videos)
* [Latest Downloadable Game Builds](http://aeplay.org/citybound-livebuilds) (for Windows, Mac and Linux)
* [Living Design Doc](https://www.notion.so/aeplay/Citybound-Living-Design-Doc-3b42707cbca54d079d301d9190ac85bb) (includes detailed notes, plans, inspirations and references)
* [LICENSE](LICENSE.txt) (AGPL)
* [Contributing & Development](CONTRIBUTING.md) (includes instructions for the custom build process)

## Server-First Modernization Commands

The repository currently supports a server-first workflow (without requiring legacy browser toolchains):

* `npm run build-server`
* `npm run test-server`
* `npm run coverage-server`

Current coverage scope includes:

* `cb_server` runtime modules
* `cb_time/src/units.rs`

And intentionally excludes legacy-heavy/generated paths:

* `cb_simulation`
* `cb_planning`
* `cb_util`
* `patches`
* `cb_time/src/actors`

## Browser Smoke Test

For a repeatable browser validation pass, run:

* `npm run smoke-browser-headless`

This command:

* builds the browser bundle
* builds the release server
* starts the local server
* loads `http://localhost:1234` in headless Chromium
* fails on uncaught browser/runtime errors or an activated in-app error overlay

Requirements:

* Playwright Chromium must already be installed locally
* if the bundled headless shell is missing, run `npx playwright install chromium`
