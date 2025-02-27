// Copyright (c) Mysten Labs, Inc.
// Modifications Copyright (c) 2024 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

import { ArrowLeft16 } from '@iota/icons';
import { type ReactNode } from 'react';
import { useNavigate } from 'react-router-dom';

import { Button } from './ButtonUI';
import { Heading } from './heading';

export interface PageTitleProps {
    title?: string;
    back?: boolean | string | (() => void);
    after?: ReactNode;
}

function PageTitle({ title = '', back, after }: PageTitleProps) {
    const navigate = useNavigate();
    const backOnClick =
        back && typeof back !== 'string'
            ? () => {
                  if (typeof back === 'function') {
                      back();
                      return;
                  }
                  navigate(-1);
              }
            : undefined;
    return (
        <div className="relative flex w-full items-center gap-5">
            {after && !back ? <div className="basis-8" /> : null}
            {back ? (
                <div className="flex h-8 items-center">
                    <Button
                        to={typeof back === 'string' ? back : undefined}
                        onClick={backOnClick}
                        size="xs"
                        before={<ArrowLeft16 className="text-base leading-none" />}
                        variant="plain"
                    />
                </div>
            ) : null}
            <div className="flex flex-1 items-center justify-center overflow-hidden">
                <Heading as="h6" variant="heading6" color="gray-90" truncate>
                    {title}
                </Heading>
            </div>
            {back ? <div className="basis-8">{after}</div> : after}
        </div>
    );
}

export default PageTitle;
