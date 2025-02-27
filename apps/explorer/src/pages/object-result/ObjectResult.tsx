// Copyright (c) Mysten Labs, Inc.
// Modifications Copyright (c) 2024 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

import { useGetObject } from '@iota/core';
import { ObjectDetailsHeader } from '@iota/icons';
import { LoadingIndicator } from '@iota/ui';
import clsx from 'clsx';
import { useParams } from 'react-router-dom';

import { ErrorBoundary, PageLayout } from '~/components';
import { Banner, PageHeader } from '~/components/ui';
import { ObjectView } from '~/pages/object-result/views/ObjectView';
import { translate, type DataType } from './ObjectResultType';
import PkgView from './views/PkgView';
import { TokenView } from './views/TokenView';

const PACKAGE_TYPE_NAME = 'Move Package';

export function ObjectResult(): JSX.Element {
    const { id: objID } = useParams();
    const { data, isPending, isError, isFetched } = useGetObject(objID!);

    if (isPending) {
        return (
            <PageLayout
                content={
                    <div className="flex w-full items-center justify-center">
                        <LoadingIndicator text="Loading data" />
                    </div>
                }
            />
        );
    }

    const isPageError = isError || data.error || (isFetched && !data);

    const resp = data && !isPageError ? translate(data) : null;
    const isPackage = resp ? resp.objType === PACKAGE_TYPE_NAME : false;

    return (
        <PageLayout
            content={
                <>
                    {isPackage ? undefined : (
                        <div>
                            <PageHeader
                                type="Object"
                                title={resp?.id ?? ''}
                                before={<ObjectDetailsHeader className="h-6 w-6" />}
                            />

                            <ErrorBoundary>
                                {data && (
                                    <div className="mt-5">
                                        <ObjectView data={data} />
                                    </div>
                                )}
                            </ErrorBoundary>
                        </div>
                    )}
                    {isPageError || !data || !resp ? (
                        <Banner variant="error" spacing="lg" fullWidth>
                            Data could not be extracted on the following specified object ID:{' '}
                            {objID}
                        </Banner>
                    ) : (
                        <div className="mb-10">
                            {isPackage && <PageHeader type="Package" title={resp.id} />}
                            <ErrorBoundary>
                                <div className={clsx(isPackage && 'mt-10')}>
                                    {isPackage ? (
                                        <PkgView data={resp} />
                                    ) : (
                                        <TokenView data={data} />
                                    )}
                                </div>
                            </ErrorBoundary>
                        </div>
                    )}
                </>
            }
        />
    );
}

export type { DataType };
