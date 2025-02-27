// Copyright (c) Mysten Labs, Inc.
// Modifications Copyright (c) 2024 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

import { formatAmount, formatBalance, formatDate } from '@iota/core';
import { useIotaClientQuery } from '@iota/dapp-kit';
import { Heading, Text, LoadingIndicator } from '@iota/ui';
import { ParentSize } from '@visx/responsive';

import { AreaGraph } from './AreaGraph';
import { ErrorBoundary } from './error-boundary/ErrorBoundary';
import { LabelText, LabelTextSize, Panel, Title, TitleSize } from '@iota/apps-ui-kit';

interface TooltipContentProps {
    data: {
        epochTotalTransactions: number;
        epochStartTimestamp: number;
        epoch: number;
    };
}

function TooltipContent({
    data: { epochTotalTransactions, epochStartTimestamp, epoch },
}: TooltipContentProps): JSX.Element {
    const dateFormatted = formatDate(new Date(epochStartTimestamp), ['day', 'month']);
    const totalFormatted = formatAmount(epochTotalTransactions);
    return (
        <div className="flex flex-col gap-0.5">
            <Text variant="subtitleSmallExtra/medium" color="steel-darker">
                {dateFormatted}, Epoch {epoch}
            </Text>
            <Heading variant="heading6/semibold" color="steel-darker">
                {totalFormatted}
            </Heading>
            <Text variant="subtitleSmallExtra/medium" color="steel-darker" uppercase>
                Transaction Blocks
            </Text>
        </div>
    );
}

function useEpochTransactions() {
    return useIotaClientQuery(
        'getEpochMetrics',
        {
            descendingOrder: true,
            limit: 31,
        },
        {
            select: (data) =>
                data.data
                    .map(({ epoch, epochTotalTransactions, epochStartTimestamp }) => ({
                        epoch: Number(epoch),
                        epochTotalTransactions: Number(epochTotalTransactions),
                        epochStartTimestamp: Number(epochStartTimestamp),
                    }))
                    .reverse()
                    .slice(0, -1),
        },
    );
}

export function TransactionsCardGraph() {
    const { data: totalTransactions } = useIotaClientQuery(
        'getTotalTransactionBlocks',
        {},
        {
            gcTime: 24 * 60 * 60 * 1000,
            staleTime: Infinity,
            retry: 5,
        },
    );
    const { data: epochMetrics, isPending } = useEpochTransactions();

    const lastEpochTotalTransactions =
        epochMetrics?.[epochMetrics.length - 1]?.epochTotalTransactions;

    return (
        <Panel>
            <Title title="Transaction Blocks" size={TitleSize.Medium} />
            <div className="flex h-full flex-col gap-md p-md--rs">
                <div className="flex flex-row gap-md">
                    <div className="flex-1">
                        <LabelText
                            size={LabelTextSize.Large}
                            label="Total"
                            text={totalTransactions ? formatBalance(totalTransactions, 0) : '--'}
                            showSupportingLabel={false}
                        />
                    </div>

                    <div className="flex-1">
                        <LabelText
                            size={LabelTextSize.Large}
                            label="Last epoch"
                            text={
                                lastEpochTotalTransactions
                                    ? lastEpochTotalTransactions.toString()
                                    : '--'
                            }
                            showSupportingLabel={false}
                        />
                    </div>
                </div>
                <div className="flex min-h-[340px] flex-1 flex-col items-center justify-center rounded-xl transition-colors">
                    {isPending ? (
                        <div className="flex flex-col items-center gap-1">
                            <LoadingIndicator />
                            <Text color="steel" variant="body/medium">
                                loading data
                            </Text>
                        </div>
                    ) : epochMetrics?.length ? (
                        <div className="relative flex-1 self-stretch">
                            <ErrorBoundary>
                                <ParentSize className="absolute">
                                    {({ height, width }) => (
                                        <AreaGraph
                                            data={epochMetrics}
                                            height={height}
                                            width={width}
                                            getX={({ epoch }) => Number(epoch)}
                                            getY={({ epochTotalTransactions }) =>
                                                Number(epochTotalTransactions)
                                            }
                                            formatY={formatAmount}
                                            tooltipContent={TooltipContent}
                                        />
                                    )}
                                </ParentSize>
                            </ErrorBoundary>
                        </div>
                    ) : (
                        <div className="flex flex-col items-center gap-1">
                            <LoadingIndicator />
                            <Text color="steel" variant="body/medium">
                                No historical data available
                            </Text>
                        </div>
                    )}
                </div>
            </div>
        </Panel>
    );
}
