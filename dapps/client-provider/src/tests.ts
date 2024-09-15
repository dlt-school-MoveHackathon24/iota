// =============================================================================
// Imports
// =============================================================================

import * as fs from 'fs';

import { ClientProvider } from "./client-provider";
import {CtfLuckynumberInterface} from "./CtfLuckynumberInterface";

// =============================================================================


// =============================================================================
// Main
// =============================================================================

const passphrase = fs.readFileSync("./passphrase.sk").toString('utf-8');

// TODO: read from config file <-> config guide on the README.md
const cp = new ClientProvider<CtfLuckynumberInterface>({
    rpcUrl: "https://api.hackanet.iota.cafe/",
    package: "com.example",
    module: "CtfLuckynumber",
    passphrase: passphrase
});

cp.invoke("get_flag", { user_counter: {id: "sss"}, lucky_num: 1 });

// =============================================================================