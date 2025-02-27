// Copyright (c) Mysten Labs, Inc.
// Modifications Copyright (c) 2024 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

import { getRefGasPrice } from '@iota/core';
import { useIotaClientQuery } from '@iota/dapp-kit';
import { Heading, Text } from '@iota/ui';
import { useMemo } from 'react';

import { Card, RingChart, RingChartLegend } from '~/components/ui';

export function ValidatorStatus(): JSX.Element | null {
    const { data } = useIotaClientQuery('getLatestIotaSystemState');

    const nextRefGasPrice = useMemo(
        () => getRefGasPrice(data?.activeValidators),
        [data?.activeValidators],
    );

    if (!data) return null;

    const nextEpoch = Number(data.epoch || 0) + 1;

    const chartData = [
        {
            value: data.activeValidators.length,
            label: 'Active',
            gradient: {
                deg: 315,
                values: [
                    { percent: 0, color: '#4C75A6' },
                    { percent: 100, color: '#589AEA' },
                ],
            },
        },
        {
            value: Number(data.pendingActiveValidatorsSize ?? 0),
            label: 'New',
            color: '#F2BD24',
        },
        {
            value: data.atRiskValidators.length,
            label: 'At Risk',
            color: '#FF794B',
        },
    ];

    return (
        <Card spacing="lg" bg="white" border="steel" rounded="2xl">
            <div className="flex items-center gap-5">
                <div className="min-h-[96px] min-w-[96px]">
                    <RingChart data={chartData} />
                </div>

                <div className="self-start">
                    <RingChartLegend data={chartData} title={`Validators in Epoch ${nextEpoch}`} />
                </div>
            </div>

            <div className="mt-8 flex items-center justify-between rounded-lg border border-solid border-steel px-3 py-2">
                <div>
                    <Text variant="pSubtitle/semibold" color="steel-darker">
                        Estimated Next Epoch
                    </Text>
                    <Text variant="pSubtitle/semibold" color="steel-darker">
                        Reference Gas Price
                    </Text>
                </div>
                <div className="text-right">
                    <Heading variant="heading4/semibold" color="steel-darker">
                        {nextRefGasPrice.toString()}
                    </Heading>
                    <Text variant="pBody/medium" color="steel-darker">
                        nano
                    </Text>
                </div>
            </div>
        </Card>
    );
}
