// Required imports
const { ApiPromise, WsProvider } = require("@polkadot/api");
const { ISubmittableResult, EventRecord } = require("@polkadot/types/types");
const { EraRewardPoints } = require("@polkadot/types/interfaces/staking");
const { Keyring, KeyringPair } = require("@polkadot/keyring");
const { randomAsU8a } = require("@polkadot/util-crypto");

const WS_URL = process.env.WS_URL;
const PHRASE = process.env.PHRASE;

async function main() {
    // Initialise the provider to connect to the local node
    const provider = new WsProvider(WS_URL);

    const types = require("../config/types.json");

    // Create the API and wait until ready
    const api = await ApiPromise.create({ provider, types });

    const keyring = new Keyring({ type: "sr25519" });
    const account = keyring.addFromUri(PHRASE);

    // Retrieve the chain & node information information via rpc calls
    const [chain, nodeName, nodeVersion] = await Promise.all([
        api.rpc.system.chain(),
        api.rpc.system.name(),
        api.rpc.system.version(),
    ]);

    console.log(`You are connected to chain ${chain} using ${nodeName} v${nodeVersion}`);

    const [historyDepth, currentEra] = await Promise.all([
        api.query.staking.historyDepth(),
        api.query.staking.currentEra(),
    ]);

    console.log(`Got init data from storage, historyDepth: ${historyDepth}, currentEra: ${currentEra}`);

    let { nonce } = await api.query.system.account(account.address);
    for (let era = Math.max(currentEra - historyDepth, 0); era < currentEra; era ++) {
        const erasRewardPoints: typeof EraRewardPoints = await api.query.staking.erasRewardPoints(era);
        for (const stash in JSON.parse(erasRewardPoints.individual)) {
            console.log(`Start processing, era: ${era}, stash: ${stash}, nonce: ${nonce}}`);
            await payout_stakers(api, account, era, stash, nonce);
            nonce++;
        }
    }

    sleep(1000);
    console.log("Done");
}

async function payout_stakers(
    api: typeof ApiPromise,
    account: typeof KeyringPair,
    era: number,
    stash: string,
    nonce: number,
) {
    await api.tx.socialTreasury
        .payoutStakers(stash, era)
        .signAndSend(account, { nonce }, ({ events = [], status, dispatchError}: typeof ISubmittableResult) => {
            console.log("Transaction status:", status.type);

            if (dispatchError) {
                if (dispatchError.isModule) {
                    // for module errors, we have the section indexed, lookup
                    const decoded = api.registry.findMetaError(dispatchError.asModule);
                    const { documentation, name, section } = decoded;
                    console.log(`${section}.${name}: ${documentation.join(" ")}`);
                } else {
                    // Other, CannotLookup, BadOrigin, no extra info
                    console.log(dispatchError.toString());
                }
            }

            if (status.isInBlock) {
                console.log("Included at block hash", status.asInBlock.toHex());
                console.log("Events:");

                events.forEach(({ event: { data, method, section }, phase }: typeof EventRecord) => {
                    console.log("\t", phase.toString(), `: ${section}.${method}`, data.toString());
                });
            } else if (status.isFinalized) {
                console.log("Finalized block hash", status.asFinalized.toHex());
            }
        });
}

function sleep(ms: number) {
    return new Promise((resolve) => setTimeout(resolve, ms));
}

function check_config() {
    if (WS_URL == null) {
        console.log("Please set WS_URL");
        process.exit(1);
    }
    if (PHRASE == null) {
        console.log("Please set PHRASE");
        process.exit(1);
    }
}

check_config();
main().catch(console.error).finally(() => process.exit());
