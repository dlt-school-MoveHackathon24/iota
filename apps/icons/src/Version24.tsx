// Copyright (c) Mysten Labs, Inc.
// Modifications Copyright (c) 2024 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0
import { SVGProps } from 'react';
const SvgVersion24 = (props: SVGProps<SVGSVGElement>) => (
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
            d="M12 18.86c3.829 0 7-3.134 7-6.93C19 8.14 15.829 5 11.993 5 8.165 5 5 8.14 5 11.93c0 3.796 3.171 6.93 7 6.93Zm0-1.363c-3.117 0-5.618-2.482-5.618-5.567 0-3.086 2.494-5.562 5.611-5.562 3.117 0 5.631 2.476 5.631 5.562 0 3.085-2.507 5.567-5.624 5.567ZM9.71 10.44l1.944 1.919a.536.536 0 0 0 .766 0l2.995-2.965a.515.515 0 0 0-.006-.751.546.546 0 0 0-.76 0l-2.615 2.596-1.559-1.55a.528.528 0 0 0-.759 0 .527.527 0 0 0-.006.751Zm-4.642 5.313v1.952c0 .88.433 1.295 1.328 1.295h2.107c-1.49-.724-2.622-1.798-3.435-3.247Zm13.918 0A7.729 7.729 0 0 1 15.551 19h2.107c.895 0 1.328-.416 1.328-1.295v-1.952Z"
        />
    </svg>
);
export default SvgVersion24;
