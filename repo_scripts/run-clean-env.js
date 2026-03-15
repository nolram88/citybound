const { spawnSync } = require("child_process");

const args = process.argv.slice(2);

if (args.length === 0) {
    console.error("Usage: node repo_scripts/run-clean-env.js <command> [args...]");
    process.exit(1);
}

const [command, ...commandArgs] = args;
const env = { ...process.env };
delete env.CARGO;
delete env.RUSTUP_TOOLCHAIN;

const result = spawnSync(command, commandArgs, {
    env,
    stdio: "inherit",
});

if (result.error) {
    console.error(result.error.message);
    process.exit(1);
}

process.exit(result.status === null ? 1 : result.status);
