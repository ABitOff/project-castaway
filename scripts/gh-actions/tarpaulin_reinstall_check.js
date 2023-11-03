const cp = require("child_process");

/**
 * Promisified version of `child_process.exec`. See it for more info.
 *
 * @param {string} command
 * @returns {Promise<{error: cp.ExecException | null, stdout: string, stderr: string}>}
 */
const exec = async (command) =>
    new Promise((res) => cp.exec(command, (error, stdout, stderr) => res({ error, stdout, stderr })));

/** Command used to test tarpaulin's current version */
const CURRENT_VERSION_CMD = "cargo tarpaulin --version";
/** Desired tarpaulin version */
const REQUESTED_VERSION = "0.27.1";
/** Command used to force an update to tarpaulin */
const FORCE_UPDATE_CMD = "cargo install --force cargo-tarpaulin";

/**
 * Asynchronously executes the specified command. Echos stdout and stderr, returns stdout. Errors
 * if the child_process errors.
 *
 * @param {string} cmd
 * @returns {Promise<string>}
 */
async function runCMD(cmd) {
    console.log(`> ${cmd}`);
    const res = await exec(cmd);

    if (res.stdout && res.stdout.length > 0) console.log(res.stdout);
    if (res.stderr && res.stderr.length > 0) console.error(res.stderr);
    if (res.error) throw res.error;

    return res.stdout;
}

(async () => {
    // run the command to get the current version. look for something like "1.2.3" in stdout. if we
    // find something, assume that it's the current version. it'd be really dumb and annoying if it
    // wasn't the current version for some reason... if we get an error, it's likely that tarpaulin
    // isn't installed, so we just fall through and do a force install.
    let currentVersion;
    try {
        currentVersion = (await runCMD(CURRENT_VERSION_CMD))?.match(/\d+\.\d+\.\d+/)?.[0];
        if (!currentVersion) {
            // no version string, versions won't match, force install.
            console.error(`Could not find tarpaulin's current version number.`);
        } else {
            // we found some kinda version string...
            console.log(`Detected current version: ${currentVersion}`);
        }
    } catch (_) {}

    // if the versions don't match, force update. otherwise, there's nothing we need to do.
    if (currentVersion != REQUESTED_VERSION) {
        console.log("Versions don't match! Forcing update.");
        await runCMD(FORCE_UPDATE_CMD);
    } else {
        console.log("Versions match. No update needed.");
    }
})();
