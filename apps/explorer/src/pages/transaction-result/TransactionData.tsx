// Copyright (c) Mysten Labs, Inc.
// Modifications Copyright (c) 2024 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

import { useTransactionSummary } from '@iota/core';
import {
    type ProgrammableTransaction,
    type IotaTransactionBlockResponse,
} from '@iota/iota-sdk/client';

import { TransactionDetailCard } from './transaction-summary/TransactionDetailCard';
import { GasBreakdown } from '~/components';
import { useRecognizedPackages } from '~/hooks/useRecognizedPackages';
import { InputsCard } from '~/pages/transaction-result/programmable-transaction-view/InputsCard';
import { TransactionsCard } from '~/pages/transaction-result/programmable-transaction-view/TransactionsCard';

interface TransactionDataProps {
    transaction: IotaTransactionBlockResponse;
}

export function TransactionData({ transaction }: TransactionDataProps): JSX.Element {
    const recognizedPackagesList = useRecognizedPackages();
    const summary = useTransactionSummary({
        transaction,
        recognizedPackagesList,
    });

    const transactionKindName = transaction.transaction?.data.transaction.kind;

    const isProgrammableTransaction = transactionKindName === 'ProgrammableTransaction';

    const programmableTxn = transaction.transaction!.data.transaction as ProgrammableTransaction;

    return (
        <div className="flex flex-wrap gap-3 pl-1 pr-2 md:gap-6">
            <section className="flex w-96 flex-1 flex-col gap-3 max-md:min-w-[50%] md:gap-6">
                <TransactionDetailCard
                    timestamp={summary?.timestamp}
                    sender={summary?.sender}
                    checkpoint={transaction.checkpoint}
                    executedEpoch={transaction.effects?.executedEpoch}
                />

                {isProgrammableTransaction && (
                    <div data-testid="inputs-card">
                        <InputsCard inputs={programmableTxn.inputs} />
                    </div>
                )}
            </section>

            {isProgrammableTransaction && (
                <section className="flex w-96 flex-1 flex-col gap-3 md:min-w-transactionColumn md:gap-6">
                    <div data-testid="transactions-card">
                        <TransactionsCard transactions={programmableTxn.transactions} />
                    </div>
                    <div data-testid="gas-breakdown">
                        <GasBreakdown summary={summary} />
                    </div>
                </section>
            )}
        </div>
    );
}
