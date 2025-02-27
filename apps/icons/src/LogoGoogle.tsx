// Copyright (c) Mysten Labs, Inc.
// Modifications Copyright (c) 2024 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0
import { SVGProps } from 'react';
const SvgLogoGoogle = (props: SVGProps<SVGSVGElement>) => (
    <svg
        xmlns="http://www.w3.org/2000/svg"
        width="1em"
        height="1em"
        fill="none"
        viewBox="0 0 16 16"
        {...props}
    >
        <path
            fill="#4285F4"
            d="M15.68 8.178c0-.658-.053-1.138-.169-1.636h-7.51v2.97h4.408c-.089.737-.569 1.848-1.636 2.595l-.015.099 2.375 1.84.165.016c1.51-1.395 2.382-3.449 2.382-5.884Z"
        />
        <path
            fill="#34A853"
            d="M8 16c2.16 0 3.973-.711 5.298-1.938l-2.525-1.955c-.675.47-1.582.8-2.773.8-2.115 0-3.911-1.396-4.551-3.325l-.094.008-2.47 1.911-.032.09A7.994 7.994 0 0 0 8 16Z"
        />
        <path
            fill="#FBBC05"
            d="M3.449 9.582A4.925 4.925 0 0 1 3.182 8c0-.551.098-1.084.258-1.582l-.004-.106-2.5-1.942-.083.039A8.007 8.007 0 0 0 0 8c0 1.289.311 2.507.853 3.591L3.45 9.582Z"
        />
        <path
            fill="#EB4335"
            d="M8 3.093c1.502 0 2.516.65 3.093 1.191l2.258-2.204C11.964.791 10.16 0 8 0A7.994 7.994 0 0 0 .853 4.409L3.44 6.418C4.089 4.488 5.885 3.093 8 3.093Z"
        />
    </svg>
);
export default SvgLogoGoogle;
