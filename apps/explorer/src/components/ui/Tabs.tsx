// Copyright (c) Mysten Labs, Inc.
// Modifications Copyright (c) 2024 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

import * as TabsPrimitive from '@radix-ui/react-tabs';
import { cva, type VariantProps } from 'class-variance-authority';
import clsx from 'clsx';
import {
    type ComponentPropsWithoutRef,
    type ElementRef,
    forwardRef,
    createContext,
    useContext,
    type ReactNode,
} from 'react';

import { Tooltip } from './Tooltip';
import { ReactComponent as InfoSvg } from './icons/info_10x10.svg';
import { ampli } from '~/lib/utils';

type TabSize = 'md' | 'lg' | 'sm';

const TabSizeContext = createContext<TabSize | null | undefined>(null);

const tabStyles = cva(
    [
        'flex items-center gap-1 border-b border-transparent -mb-px',
        'text-neutral-60 disabled:text-steel-dark disabled:pointer-events-none hover:text-neutral-10 data-[state=active]:border-primary-30',
    ],
    {
        variants: {
            size: {
                lg: 'text-heading4 data-[state=active]:text-neutral-10 pb-4',
                md: 'text-body data-[state=active]:text-neutral-10 pb-4',
                sm: 'text-captionSmall font-medium pb-1 disabled:opacity-40 data-[state=active]:text-neutral-10',
            },
        },
        defaultVariants: {
            size: 'md',
        },
    },
);
const tabListStyles = cva(['flex items-center border-gray-45'], {
    variants: {
        fullWidth: {
            true: 'flex-1',
        },
        disableBottomBorder: {
            true: '',
            false: 'border-b',
        },
        gap: {
            3: 'gap-3',
            6: 'gap-4 sm:gap-6',
        },
    },
    defaultVariants: {
        gap: 6,
        disableBottomBorder: false,
    },
});

type TabListStylesProps = VariantProps<typeof tabListStyles>;

const Tabs = forwardRef<
    ElementRef<typeof TabsPrimitive.Root>,
    ComponentPropsWithoutRef<typeof TabsPrimitive.Root> & { size?: TabSize }
>(({ size, ...props }, ref) => (
    <TabSizeContext.Provider value={size}>
        <TabsPrimitive.Root ref={ref} {...props} />
    </TabSizeContext.Provider>
));

const TabsList = forwardRef<
    ElementRef<typeof TabsPrimitive.List>,
    ComponentPropsWithoutRef<typeof TabsPrimitive.List> & TabListStylesProps
>(({ fullWidth, disableBottomBorder, gap, ...props }, ref) => (
    <TabsPrimitive.List
        ref={ref}
        className={tabListStyles({ fullWidth, disableBottomBorder, gap })}
        {...props}
    />
));

const TabsTrigger = forwardRef<
    ElementRef<typeof TabsPrimitive.Trigger>,
    ComponentPropsWithoutRef<typeof TabsPrimitive.Trigger>
>(({ className, ...props }, ref) => {
    const size = useContext(TabSizeContext);

    return (
        <TabsPrimitive.Trigger
            ref={ref}
            className={clsx(tabStyles({ size }), className)}
            {...props}
        />
    );
});

const TabsContent = forwardRef<
    ElementRef<typeof TabsPrimitive.Content>,
    ComponentPropsWithoutRef<typeof TabsPrimitive.Content> & { noGap?: boolean }
>(({ noGap, ...props }, ref) => (
    <TabsPrimitive.Content
        ref={ref}
        className={clsx(
            'ring-offset-background focus-visible:ring-ring focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-offset-2',
            !noGap && 'my-4',
        )}
        {...props}
    />
));

export { Tabs, TabsList, TabsTrigger, TabsContent };

/**
 * A special single-tab header that automatically creates the correct components and state.
 * TODO: This probably shouldn't even be tabs, because that's bad for a11y when there's just single tabs acting as headers.
 * We should instead just re-define this as a header components.
 */

interface TabHeaderProps {
    size?: TabSize;
    title: string;
    children: ReactNode;
    noGap?: boolean;
    tooltip?: string;
}

export function TabHeader({
    size = 'lg',
    title,
    children,
    noGap,
    tooltip,
}: TabHeaderProps): JSX.Element {
    return (
        <Tabs size={size} defaultValue="tab">
            <TabsList>
                <TabsTrigger value="tab">
                    {title}
                    {tooltip && (
                        <Tooltip
                            tip={tooltip}
                            onOpen={() => {
                                ampli.activatedTooltip({ tooltipLabel: title });
                            }}
                        >
                            <InfoSvg />
                        </Tooltip>
                    )}
                </TabsTrigger>
            </TabsList>
            <TabsContent value="tab" noGap={noGap}>
                {children}
            </TabsContent>
        </Tabs>
    );
}
