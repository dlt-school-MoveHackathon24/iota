// Copyright (c) Mysten Labs, Inc.
// Modifications Copyright (c) 2024 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

import { type IotaTransactionBlockResponse } from '@iota/iota-sdk/client';
import clsx from 'clsx';
import { type ReactNode, useState } from 'react';

import { ErrorBoundary } from '~/components';
import { SplitPanes, Tabs, TabsContent, TabsList, TabsTrigger } from '~/components/ui';
import { useBreakpoint } from '~/hooks/useBreakpoint';
import { Events } from '~/pages/transaction-result/Events';
import { TransactionData } from '~/pages/transaction-result/TransactionData';
import { TransactionSummary } from '~/pages/transaction-result/transaction-summary';
import { Signatures } from './Signatures';

import { LocalStorageSplitPaneKey } from '~/lib/enums';
import styles from './TransactionResult.module.css';

interface TabsContentContainerProps {
    value: string;
    children: ReactNode;
}

function TabsContentContainer({ value, children }: TabsContentContainerProps): JSX.Element {
    return (
        <TabsContent value={value}>
            <div className="mt-6 md:mt-10">{children}</div>
        </TabsContent>
    );
}

interface TransactionViewProps {
    transaction: IotaTransactionBlockResponse;
}

export function TransactionView({ transaction }: TransactionViewProps): JSX.Element {
    const isMediumOrAbove = useBreakpoint('md');
    const [isCollapsed, setIsCollapsed] = useState(false);

    const hasEvents = !!transaction.events?.length;

    const transactionKindName = transaction.transaction?.data.transaction?.kind;

    const isProgrammableTransaction = transactionKindName === 'ProgrammableTransaction';

    const leftPane = {
        panel: (
            <div className="h-full overflow-y-auto rounded-2xl border border-transparent bg-gray-40 p-6 md:h-full md:max-h-screen md:p-10">
                <Tabs size="lg" defaultValue="summary">
                    <TabsList>
                        <TabsTrigger value="summary">Summary</TabsTrigger>
                        {hasEvents && <TabsTrigger value="events">Events</TabsTrigger>}
                        {isProgrammableTransaction && (
                            <TabsTrigger value="signatures">Signatures</TabsTrigger>
                        )}
                    </TabsList>
                    <TabsContentContainer value="summary">
                        <TransactionSummary transaction={transaction} />
                    </TabsContentContainer>
                    {hasEvents && (
                        <TabsContentContainer value="events">
                            <Events events={transaction.events!} />
                        </TabsContentContainer>
                    )}
                    <TabsContentContainer value="signatures">
                        <ErrorBoundary>
                            <Signatures transaction={transaction} />
                        </ErrorBoundary>
                    </TabsContentContainer>
                </Tabs>
            </div>
        ),
        minSize: 35,
        collapsible: true,
        collapsibleButton: true,
        noHoverHidden: isMediumOrAbove,
    };

    const rightPane = {
        panel: (
            <div
                className={clsx(
                    'h-full w-full overflow-y-auto md:overflow-y-hidden',
                    isCollapsed && isMediumOrAbove && 'pl-2',
                )}
            >
                <TransactionData transaction={transaction} />
            </div>
        ),
        minSize: 40,
        defaultSize: isProgrammableTransaction ? 65 : 50,
    };

    return (
        <div className={clsx(styles.txdetailsbg)}>
            <div className="h-screen md:h-full">
                <SplitPanes
                    autoSaveId={LocalStorageSplitPaneKey.TransactionView}
                    onCollapse={setIsCollapsed}
                    dividerSize={isMediumOrAbove ? 'md' : 'lg'}
                    splitPanels={[leftPane, rightPane]}
                    direction={isMediumOrAbove ? 'horizontal' : 'vertical'}
                />
            </div>
        </div>
    );
}
