// Copyright (c) Mysten Labs, Inc.
// Modifications Copyright (c) 2024 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

import { X12 } from '@iota/icons';
import { type Meta, type StoryObj } from '@storybook/react';

import { IconButton } from './IconButton';

const meta = {
    component: IconButton,
} satisfies Meta<typeof IconButton>;

export default meta;

type Story = StoryObj<typeof meta>;

export const Default: Story = {
    args: { 'aria-label': 'Close', children: <X12 /> },
};

export const Disabled: Story = {
    args: { ...Default.args, disabled: true },
};

export const AsChild: Story = {
    args: {
        ...Default.args,
        children: (
            <a href="https://google.com">
                <X12 />
            </a>
        ),
        asChild: true,
    },
};
