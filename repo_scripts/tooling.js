const util = require('util');
const { execSync, spawnSync } = require('child_process');

const NIGHTLY_VERSION = "nightly";
const WASM_BINDGEN_VERSION = "0.2.100";
const args = new Set(process.argv.slice(2));
let quiet = args.has("-q") || args.has("--quiet");
const serverOnly = args.has("--server");
const browserOnly = args.has("--browser");
const toolingEnv = Object.assign({}, process.env);
delete toolingEnv.RUSTUP_TOOLCHAIN;
delete toolingEnv.CARGO;

function exec(command, options = {}) {
    return execSync(
        command,
        Object.assign({ encoding: 'utf8', env: toolingEnv }, options)
    );
}

function spawn(command, commandArgs, options = {}) {
    return spawnSync(
        command,
        commandArgs,
        Object.assign({ env: toolingEnv }, options)
    );
}

if (serverOnly && browserOnly) {
    console.log("Choose only one mode: --server or --browser");
    process.exit(1);
}

const shouldCheckServer = !browserOnly;
const shouldCheckBrowser = !serverOnly;

let rustupV;
try {
    rustupV = exec('rustup --version');
} catch (e) {
    rustupV = undefined
}

if (rustupV && rustupV.startsWith("rustup")) {
    !quiet && console.log("Rustup installed ✅ (OK)");
} else {
    console.log("Rustup missing! 🛑 (FAIL)");
    console.log("Please install from https://rustup.rs");
    process.exit(1);
}

function ensureRustNightly(nightly) {
    let activeToolchain = exec('rustup show active-toolchain')
        .split(/\s+/)[0]
        .trim();

    if (activeToolchain && activeToolchain.includes(nightly)) {
        !quiet && console.log("Correct rust nightly set up ✅ (OK)");
    } else {
        let rustupShow = exec('rustup show');
        console.log("Wrong version of rust set up ❗️(!)");
        console.log(rustupShow.split(/\n/g).map(s => " | " + s).join("\n"));
        console.log("🔧 Overriding with correct nightly (only for this directory)...");
        const defaultHost =
            (rustupShow.match(/Default host:\s+([^\n\r]+)/) || [null, null])[1];
        const fullVersion = defaultHost ? `${nightly}-${defaultHost}` : nightly;
        console.log("> rustup override set " + fullVersion);
        spawn("rustup", ["override", "set", fullVersion], { stdio: 'inherit' });
        quiet = false;

        let activeToolchain2 = exec('rustup show active-toolchain')
            .split(/\s+/)[0]
            .trim();
        if (activeToolchain2 && activeToolchain2.includes(nightly)) {
            !quiet && console.log("Correct rust nightly set up ✅ (OK)");
        } else {
            let rustupShow2 = exec('rustup show');
            console.log("Failed to install correct toolchain 🛑 (FAIL)");
            console.log("rustup show output:");
            console.log(rustupShow2);
            process.exit(1);
        }
    }
}

if (shouldCheckServer) {
    !quiet && console.log("Checking rust nightly for simulation");
    ensureRustNightly(NIGHTLY_VERSION);
}

if (shouldCheckBrowser) {
    process.chdir('./cb_browser_ui');

    !quiet && console.log("Checking rust nightly for browser");
    ensureRustNightly(NIGHTLY_VERSION);

    !quiet && console.log("Ensuring wasm32 target is installed");
    spawn("rustup", ["target", "add", "wasm32-unknown-unknown", "--toolchain", NIGHTLY_VERSION],
        { stdio: quiet ? 'ignore' : 'inherit' }
    );

    !quiet && console.log("Checking wasm-bindgen version");

    function checkWasmBindgen(requiredVersion) {
        try {
            let wasmBindgenVersion = exec('wasm-bindgen --version');
            return wasmBindgenVersion.includes(requiredVersion);
        } catch (e) {
            console.log("Couldn't run wasm-bindgen", e.message);
            return false;
        }
    }

    if (checkWasmBindgen(WASM_BINDGEN_VERSION)) {
        !quiet && console.log("Correct wasm-bindgen set up ✅ (OK)");
    } else {
        !quiet && console.log("Correct wasm-bindgen not installed yet ❗️(!)");
        console.log("🔧 Installing wasm-bindgen-cli");
        spawn("cargo", ["install", "wasm-bindgen-cli", "--force", "--version", WASM_BINDGEN_VERSION],
            { stdio: quiet ? 'ignore' : 'inherit' }
        );

        if (checkWasmBindgen(WASM_BINDGEN_VERSION)) {
            !quiet && console.log("Correct wasm-bindgen set up ✅ (OK)");
        } else {
            console.log("Failed to install wasm-bindgen 🛑 (FAIL)");
            process.exit(1);
        }
    }

    process.chdir('..');
}

if (shouldCheckServer) {
    !quiet && console.log("🔧 Ensuring linting tools are installed...");
    spawn("rustup", ["component", "add", "rustfmt-preview", "--toolchain", NIGHTLY_VERSION],
        { stdio: quiet ? 'ignore' : 'inherit' }
    );
    spawn("rustup", ["component", "add", "clippy-preview", "--toolchain", NIGHTLY_VERSION],
        { stdio: quiet ? 'ignore' : 'inherit' }
    );
    !quiet && console.log("Linting tools set up ✅ (OK)");
}
