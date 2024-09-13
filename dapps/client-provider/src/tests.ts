import { ClientProvider } from "./client-provider";
import {CtfLuckynumberInterface} from "./CtfLuckynumberInterface";

const cp = new ClientProvider<CtfLuckynumberInterface>({
    rpcUrl: "http://localhost:8080",
    package: "com.example",
    module: "CtfLuckynumber",
    privateKey: "ANA0S10DPSAE9egA26TVS6Oi4FJCRTs6LrzxtDCdbOXh"
});

cp.invoke("get_flag", { user_counter: {id: "sss"}, lucky_num: 1 });