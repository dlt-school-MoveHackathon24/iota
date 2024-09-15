// =============================================================================
// Imports
// =============================================================================

import { ClientProvider } from "./client-provider";
import {CtfLuckynumberInterface} from "./CtfLuckynumberInterface";

// =============================================================================


// =============================================================================
// Main
// =============================================================================

// TODO: read from config file <-> config guide on the README.md
const cp = new ClientProvider<CtfLuckynumberInterface>({
    rpcUrl: "https://api.hackanet.iota.cafe/",
    package: "com.example",
    module: "CtfLuckynumber", // ?
    privateKey: "ANA0S10DPSAE9egA26TVS6Oi4FJCRTs6LrzxtDCdbOXh" // TODO: fix this with their SDK ;D
});

cp.invoke("get_flag", { user_counter: {id: "sss"}, lucky_num: 1 });

// =============================================================================