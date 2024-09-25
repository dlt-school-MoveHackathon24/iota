import { IotaClient, IotaTransactionBlockResponse} from '@iota/iota-sdk/client';
import { requestIotaFromFaucetV1 } from '@iota/iota-sdk/faucet';
import { Inputs, TransactionBlock, TransactionObjectArgument, TransactionResult } from '@iota/iota-sdk/transactions';
import { Ed25519Keypair } from '@iota/iota-sdk/keypairs/ed25519';



//node --no-warnings --loader ts-node/esm main.ts


const COMPLETE_URI = "angelo.CS.unipg.iota";
const MNEMONIC = "drop erupt possible insect despair ski grief twice firm diagram smooth fancy";

async function sleep(ms: number): Promise<void> {
    return new Promise(
        (resolve) => setTimeout(resolve, ms));
}

async function getAddress (URI:string) {
    console.log(">> SETTING UP ACCOUNT");

    console.info(">>>> CREATING KEYPAIR");
    const mnemonic = MNEMONIC
    //const signer = Ed25519Keypair.deriveKeypair(mnemonic);
    const signer = new Ed25519Keypair();
    console.info(">>>> ADDRESS : " + signer.toIotaAddress());

    console.info(">> SETTING UP CLIENT : ");
    const client : IotaClient = new IotaClient({ url: 'https://api.hackanet.iota.cafe/' });
    
    try {
        const faucet= await requestIotaFromFaucetV1({
            host: "https://faucet.hackanet.iota.cafe/gas",
            recipient: signer.toIotaAddress(),
        });
        console.info(">> FAUCET TOTAL : " + faucet);
    } catch (error) {
        console.info("!! FAUCET ERROR : " + error);
    }

    await sleep(5000);

    console.info(">> CREATING TRANSACTION BLOCK");
    const txb : TransactionBlock = new TransactionBlock();

    const packageId : string = '0x4b8263686e699ced60293a4e7d3c9477b3f12777e0714d966e05404c666a23e5';


    console.info(">> SPLITTING URI");
    let URIsplitted : string[] = URI.split('.');
    URIsplitted.pop();
    URIsplitted = URIsplitted.reverse();
    console.info(">>>> SPLITTED URI : " + URIsplitted.toString());
    let URIobjects = [];
    URIsplitted.forEach(e => {
        URIobjects.push(txb.pure.string(e));
    });
    

    let root_id = txb.object('0xed41679eda4c1406f9f7fe2b3c873969fd6589f47c1411a87c7694d5b7a58459');
    console.info(">> CREATING VECTOR");
    const uriVector : TransactionResult = txb.makeMoveVec({type: "0x1::string::String", objects: URIobjects});
    
    console.info(">> CREATING MOVE CALL");
    txb.moveCall({
        arguments: [root_id, uriVector],
        target: `${packageId}::iotaidtest::get_address`
    });
    
    console.info(">> SETTING UP GAS BUDGET");
    txb.setGasBudget(50000000);


    console.info(">> EXECUTING TRANSACTION BLOCK");
    try {
        const TXBRES : IotaTransactionBlockResponse = await client.signAndExecuteTransactionBlock({
            transactionBlock: txb,
            signer: signer,
        });
        console.info(">> TXB RESPONSE : " + TXBRES.digest);
    } catch (error) {
        console.info("!! TXB ERROR : " + error);
    }
    
}

await getAddress(COMPLETE_URI)
