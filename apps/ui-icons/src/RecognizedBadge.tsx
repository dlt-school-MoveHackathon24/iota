// Copyright (c) 2024 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

import { SVGProps } from 'react';
export default function SvgRecognizedBadge(props: SVGProps<SVGSVGElement>) {
    return (
        <svg
            xmlns="http://www.w3.org/2000/svg"
            width="1em"
            height="1em"
            fill="none"
            viewBox="0 0 24 24"
            {...props}
        >
            <path
                fillRule="evenodd"
                clipRule="evenodd"
                d="M14.8816 5.04319L12 2L9.11839 5.04319L4.92893 4.92893L5.04319 9.11839L2 12L5.04319 14.8816L4.92893 19.0711L9.11839 18.9568L12 22L14.8816 18.9568L19.0711 19.0711L18.9568 14.8816L22 12L18.9568 9.11839L19.0711 4.92893L14.8816 5.04319ZM16.7474 9.66436C17.1143 9.25158 17.0771 8.61951 16.6644 8.25259C16.2516 7.88567 15.6195 7.92285 15.2526 8.33564L10.6238 13.543L8.70711 11.6262C8.31658 11.2357 7.68342 11.2357 7.29289 11.6262C6.90237 12.0168 6.90237 12.6499 7.29289 13.0404L9.95956 15.7071C10.3664 16.114 11.0318 16.0944 11.4141 15.6644L16.7474 9.66436Z"
                fill="currentColor"
            />
        </svg>
    );
}
