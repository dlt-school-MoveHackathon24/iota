// Copyright (c) Mysten Labs, Inc.
// Modifications Copyright (c) 2024 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

import { CoinIcon } from '_components';

import { Text } from '../../text';

export interface CoinsStackProps {
    coinTypes: string[];
}

const MAX_COINS_TO_DISPLAY = 4;

export function CoinsStack({ coinTypes }: CoinsStackProps) {
    return (
        <div className="flex">
            {coinTypes.length > MAX_COINS_TO_DISPLAY && (
                <Text variant="bodySmall" weight="medium" color="steel-dark">
                    +{coinTypes.length - MAX_COINS_TO_DISPLAY}
                </Text>
            )}
            {coinTypes.slice(0, MAX_COINS_TO_DISPLAY).map((coinType, i) => (
                <div key={coinType} className={i === 0 ? '' : '-ml-1'}>
                    <CoinIcon coinType={coinType} />
                </div>
            ))}
        </div>
    );
}
