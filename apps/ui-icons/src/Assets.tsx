// Copyright (c) 2024 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

import { SVGProps } from 'react';
export default function SvgAssets(props: SVGProps<SVGSVGElement>) {
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
                fill="currentColor"
                fillRule="evenodd"
                clipRule="evenodd"
                d="M7 3C4.23858 3 2 5.23858 2 8V14C2 16.7614 4.23858 19 7 19C8.03783 19 8.62974 19.2628 9.60481 19.524L15.4004 21.077C18.0677 21.7917 20.8094 20.2088 21.5241 17.5414L23.077 11.7459C23.7917 9.07855 22.2088 6.33686 19.5415 5.62215L16.9438 4.9261C16.0287 3.75378 14.6024 3 13 3H7ZM17.9886 7.659C17.9796 7.52628 17.9655 7.39498 17.9464 7.26531L19.0238 7.554C20.6242 7.98283 21.574 9.62784 21.1452 11.2282L19.5922 17.0238C19.1634 18.6242 17.5184 19.5739 15.918 19.1451L14.5102 18.7679C15.5961 18.4243 16.5216 17.7199 17.1461 16.7955C17.2475 16.6454 17.341 16.4895 17.426 16.3283C17.7925 15.6329 18 14.8407 18 14V7.99996C18 7.8854 17.9962 7.7717 17.9886 7.659ZM15.4491 15.733C15.7961 15.2435 16 14.6456 16 14V8C16 6.34315 14.6569 5 13 5L7 5C5.34315 5 4 6.34315 4 8L4 14C4 15.6515 5.33449 16.9913 6.98396 17H13C14.0113 17 14.9057 16.4996 15.4491 15.733Z"
            />
        </svg>
    );
}
