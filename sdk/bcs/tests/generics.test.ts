// Copyright (c) Mysten Labs, Inc.
// Modifications Copyright (c) 2024 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

import { describe, expect, it } from 'vitest';

import { BCS, getIotaMoveConfig } from './../src/index';

describe('BCS: Generics', () => {
    it('should handle generics', () => {
        const bcs = new BCS(getIotaMoveConfig());

        bcs.registerEnumType('base::Option<T>', {
            none: null,
            some: 'T',
        });

        expect(bcs.de('base::Option<u8>', '00', 'hex')).toEqual({ none: true });
    });

    it('should handle nested generics', () => {
        const bcs = new BCS(getIotaMoveConfig());

        bcs.registerEnumType('base::Option<T>', {
            none: null,
            some: 'T',
        });

        bcs.registerStructType('base::Container<T, S>', {
            tag: 'T',
            data: 'base::Option<S>',
        });

        expect(bcs.de('base::Container<bool, u8>', '0000', 'hex')).toEqual({
            tag: false,
            data: { none: true },
        });

        bcs.registerStructType('base::Wrapper', {
            wrapped: 'base::Container<bool, u8>',
        });

        expect(bcs.de('base::Wrapper', '0000', 'hex')).toEqual({
            wrapped: {
                tag: false,
                data: { none: true },
            },
        });
    });
});
