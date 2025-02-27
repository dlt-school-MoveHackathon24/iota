// Copyright (c) Mysten Labs, Inc.
// Modifications Copyright (c) 2024 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0
import { SVGProps } from 'react';
const SvgSupport24 = (props: SVGProps<SVGSVGElement>) => (
    <svg
        xmlns="http://www.w3.org/2000/svg"
        width="1em"
        height="1em"
        fill="none"
        viewBox="0 0 24 24"
        {...props}
    >
        <path
            stroke="currentColor"
            strokeLinecap="round"
            strokeLinejoin="round"
            strokeWidth={1.5}
            d="M7.733 11.445a5.764 5.764 0 0 1 5.7-6.645 5.764 5.764 0 0 1 5.395 7.8c-.05.133-.075.199-.087.25a.65.65 0 0 0-.017.139c0 .052.006.11.02.227l.289 2.354c.031.255.047.383.005.475a.359.359 0 0 1-.185.181c-.093.04-.22.022-.472-.016l-2.281-.336c-.12-.017-.179-.026-.233-.026a.64.64 0 0 0-.144.016c-.053.01-.12.036-.256.087a5.76 5.76 0 0 1-2.915.302M8.834 19.2c2.123 0 3.845-1.773 3.845-3.96 0-2.187-1.722-3.96-3.845-3.96-2.124 0-3.845 1.773-3.845 3.96 0 .44.07.862.197 1.258.055.167.082.25.09.307.01.06.012.093.008.153-.003.058-.017.123-.046.254L4.8 19.2l2.145-.294a1.75 1.75 0 0 1 .227-.024c.054 0 .082.003.135.014.05.01.125.036.274.089.392.14.814.215 1.253.215Z"
        />
    </svg>
);
export default SvgSupport24;
