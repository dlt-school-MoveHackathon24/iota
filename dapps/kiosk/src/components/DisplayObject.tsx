// Copyright (c) Mysten Labs, Inc.
// Modifications Copyright (c) 2024 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

import { useCurrentAccount } from '@iota/dapp-kit';
import { KioskListing } from '@iota/kiosk';
import { ReactNode } from 'react';

import { DEFAULT_IMAGE } from '../utils/constants';
import { formatIota, nanoToIota } from '../utils/utils';
import { OwnedObjectType } from './Inventory/OwnedObjects';
import { ItemLockedBadge } from './Kiosk/ItemLockedBadge';

export interface DisplayObject {
    listing?: KioskListing | null;
    item: OwnedObjectType;
    children: ReactNode;
}

export function DisplayObject({ item, listing = null, children }: DisplayObject) {
    const currentAccount = useCurrentAccount();

    const price = formatIota(nanoToIota(listing?.price));

    return (
        <div className="border relative border-gray-400 overflow-hidden text-center flex justify-between flex-col rounded-lg">
            <div className="h-[275px] xl:h-[200px] overflow-hidden bg-gray-50">
                <img
                    src={item.display.image_url}
                    className="object-cover aspect-auto h-full w-full mx-auto"
                    alt="The display of the object"
                    onError={(e) => {
                        const image = e.target as HTMLImageElement;
                        image.src = DEFAULT_IMAGE;
                    }}
                ></img>
            </div>

            <div className="p-4">
                {item.display.name && (
                    <h3 className="text-clip overflow-hidden">{item.display.name}</h3>
                )}

                {item.display.description && (
                    <p className="text-sm opacity-80 text-clip overflow-hidden">
                        {item.display.description}
                    </p>
                )}

                {listing && listing.price && (
                    <div className="absolute left-2 top-2 bg-primary text-white px-2 py-1 rounded-lg">
                        {price} IOTA
                    </div>
                )}

                <p className="text-xs break-words text-gray-400 py-3">{item.type}</p>
                {item.isLocked && <ItemLockedBadge />}

                {/* button actions */}
                {currentAccount?.address ? (
                    <div className="grid lg:grid-cols-2 gap-5 mt-6">{children}</div>
                ) : (
                    <div className="mt-6 text-xs">Connect your wallet to interact</div>
                )}
            </div>
        </div>
    );
}
