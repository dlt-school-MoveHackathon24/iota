// Copyright (c) Mysten Labs, Inc.
// Modifications Copyright (c) 2024 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

import clsx from 'clsx';
import { forwardRef, type ComponentProps, type ReactNode, type Ref } from 'react';
import { Link, type LinkProps } from 'react-router-dom';

import { LoadingIndicator } from '_components';
import { Tooltip } from '../tooltip';

interface WithTooltipProps {
    title?: ReactNode;
    children: ReactNode;
}

function WithTooltip({ title, children }: WithTooltipProps) {
    if (title) {
        return <Tooltip tip={title}>{children}</Tooltip>;
    }
    return children;
}

export interface ButtonOrLinkProps
    extends Omit<Partial<LinkProps> & ComponentProps<'a'> & ComponentProps<'button'>, 'ref'> {
    loading?: boolean;
}
export const ButtonOrLink = forwardRef<HTMLAnchorElement | HTMLButtonElement, ButtonOrLinkProps>(
    ({ href, to, disabled = false, loading = false, children, title, ...props }, ref) => {
        const isDisabled = disabled || loading;
        const content = loading ? (
            <>
                <div className="invisible contents !text-transparent">{children}</div>
                <div
                    data-testid="loading-indicator"
                    className="absolute left-1/2 top-1/2 flex -translate-x-1/2 -translate-y-1/2"
                >
                    <LoadingIndicator color="inherit" />
                </div>
            </>
        ) : (
            children
        );
        const styles = loading
            ? ({ position: 'relative', textOverflow: 'clip' } as const)
            : undefined;
        // External link:
        if (href && !isDisabled) {
            return (
                <WithTooltip title={title}>
                    <a
                        ref={ref as Ref<HTMLAnchorElement>}
                        target="_blank"
                        rel="noreferrer noopener"
                        href={href}
                        {...props}
                        style={styles}
                    >
                        {content}
                    </a>
                </WithTooltip>
            );
        }
        // Internal router link:
        if (to && !isDisabled) {
            return (
                <WithTooltip title={title}>
                    <Link to={to} ref={ref as Ref<HTMLAnchorElement>} {...props} style={styles}>
                        {content}
                    </Link>
                </WithTooltip>
            );
        }
        return (
            <WithTooltip title={title}>
                <button
                    {...props}
                    className={clsx(!isDisabled && 'cursor-pointer', props.className)}
                    type={props.type || 'button'}
                    ref={ref as Ref<HTMLButtonElement>}
                    disabled={isDisabled}
                    style={styles}
                >
                    {content}
                </button>
            </WithTooltip>
        );
    },
);
