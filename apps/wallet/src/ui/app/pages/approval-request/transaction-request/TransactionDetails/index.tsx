// Copyright (c) Mysten Labs, Inc.
// Modifications Copyright (c) 2024 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

import { useTransactionData } from '_src/ui/app/hooks';
import { Tab as HeadlessTab, type TabProps } from '@headlessui/react';
import { type TransactionBlock } from '@iota/iota-sdk/transactions';

import { SummaryCard } from '../SummaryCard';
import { Command } from './Command';
import { Input } from './Input';

interface TransactionDetailsProps {
    sender?: string;
    transaction: TransactionBlock;
}

const Tab = (props: TabProps<'div'>) => (
    <HeadlessTab
        className="text-steel-darker ui-selected:border-hero ui-selected:text-hero-dark -mb-px cursor-pointer border-0 border-b border-solid border-transparent bg-transparent p-0 pb-2 text-body font-semibold outline-none"
        {...props}
    />
);

export function TransactionDetails({ sender, transaction }: TransactionDetailsProps) {
    const { data: transactionData, isPending, isError } = useTransactionData(sender, transaction);
    if (transactionData?.transactions.length === 0 && transactionData.inputs.length === 0) {
        return null;
    }
    return (
        <SummaryCard header="Transaction Details" initialExpanded>
            {isPending || isError ? (
                <div className="text-steel-darker ml-0 text-pBodySmall font-medium">
                    {isPending ? 'Gathering data...' : "Couldn't gather data"}
                </div>
            ) : transactionData ? (
                <div>
                    <HeadlessTab.Group>
                        <HeadlessTab.List className="border-gray-45 mb-6 flex gap-6 border-0 border-b border-solid">
                            {!!transactionData.transactions.length && <Tab>Transactions</Tab>}
                            {!!transactionData.inputs.length && <Tab>Inputs</Tab>}
                        </HeadlessTab.List>
                        <HeadlessTab.Panels>
                            {!!transactionData.transactions.length && (
                                <HeadlessTab.Panel className="flex flex-col gap-6">
                                    {/* TODO: Rename components: */}
                                    {transactionData.transactions.map((command, index) => (
                                        <Command key={index} command={command} />
                                    ))}
                                </HeadlessTab.Panel>
                            )}
                            {!!transactionData.inputs.length && (
                                <HeadlessTab.Panel className="flex flex-col gap-2">
                                    {transactionData.inputs.map((input, index) => (
                                        <Input key={index} input={input} />
                                    ))}
                                </HeadlessTab.Panel>
                            )}
                        </HeadlessTab.Panels>
                    </HeadlessTab.Group>
                </div>
            ) : (
                ''
            )}
        </SummaryCard>
    );
}
