import deployedObjectsJson from '../../../move/deployed_objects.json';

function getIds() {
    let bridgeId = '';
    let adminCapId = '';
    let bridgePackageId = '';
    let coinManagerId = '';
    let coinManagerTreasuryCapId = '';

    if (deployedObjectsJson.objectChanges instanceof Array) {
        deployedObjectsJson.objectChanges.forEach((change: any) => {
            const { objectType, objectId, packageId } = change;

            if (packageId) {
                bridgePackageId = packageId;
            }

            if (objectType) {
                // Identify based on objectType
                if (objectType.includes('::bridge::Bridge')) {
                    bridgeId = objectId;
                } else if (objectType.includes('::bridge::AdminCap')) {
                    adminCapId = objectId;
                } else if (objectType.includes('::coin_manager::CoinManager<')) {
                    coinManagerId = objectId;
                } else if (objectType.includes('::coin_manager::CoinManagerTreasuryCap<')) {
                    coinManagerTreasuryCapId = objectId;
                }
            }
        });

        return {
            bridgeId,
            adminCapId,
            bridgePackageId,
            coinManagerId,
            coinManagerTreasuryCapId,
        };

    } else {
        console.error('objectChanges is undefined or not an array');
    }

}

export { getIds }