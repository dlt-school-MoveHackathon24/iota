// Copyright (c) Mysten Labs, Inc.
// Modifications Copyright (c) 2024 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0
import { SVGProps } from 'react';
const SvgObjectDetailsHeader = (props: SVGProps<SVGSVGElement>) => (
    <svg
        xmlns="http://www.w3.org/2000/svg"
        width="1em"
        height="1em"
        fill="none"
        viewBox="0 0 24 24"
        {...props}
    >
        <path
            fill="url(#object_details_header_svg__a)"
            fillRule="evenodd"
            d="M4.125.5a1 1 0 0 1 1 1v1.566H6.75a1 1 0 1 1 0 2H5.125v1.566a1 1 0 0 1-2 0V5.066H1.5a1 1 0 0 1 0-2h1.625V1.5a1 1 0 0 1 1-1Zm8.925 1.026a1 1 0 0 1 .93.634l1.821 4.628c.315.8.415 1.034.553 1.223.138.19.308.356.504.492.197.137.442.237 1.26.544l4.734 1.78a1 1 0 0 1 0 1.872l-4.734 1.78c-.818.307-1.063.407-1.26.544a2.115 2.115 0 0 0-.504.492c-.138.19-.238.423-.553 1.224l-1.82 4.627a1 1 0 0 1-1.861 0l-1.821-4.627c-.315-.8-.415-1.034-.553-1.224a2.115 2.115 0 0 0-.504-.492c-.197-.137-.442-.237-1.26-.544l-4.734-1.78a1 1 0 0 1 0-1.872l4.734-1.78c.818-.307 1.063-.407 1.26-.544.196-.136.366-.303.504-.492.138-.19.238-.423.553-1.223l1.82-4.628a1 1 0 0 1 .931-.634Zm0 3.731-.89 2.263-.04.103c-.257.653-.45 1.143-.757 1.564-.27.372-.601.696-.98.958-.427.298-.924.484-1.593.735l-.104.04-2.244.843 2.244.844.104.04c.67.25 1.166.437 1.594.734.378.263.71.586.98.958.306.422.499.912.755 1.564l.04.103.891 2.263.89-2.263c.014-.034.028-.069.04-.103.257-.652.45-1.142.757-1.564.27-.372.601-.695.98-.958.427-.297.924-.484 1.593-.735l.104-.04 2.244-.843-2.244-.844-.104-.039c-.67-.25-1.166-.437-1.594-.735a4.116 4.116 0 0 1-.98-.958c-.306-.421-.499-.911-.755-1.564a30.175 30.175 0 0 0-.04-.103l-.891-2.263ZM1.846 17.08a1 1 0 0 1 1.415 0l1.122 1.123 1.123-1.123a1 1 0 0 1 1.415 1.415l-1.123 1.123 1.123 1.122a1 1 0 1 1-1.415 1.415L4.383 21.03l-1.122 1.123a1 1 0 0 1-1.415-1.415l1.123-1.122-1.123-1.123a1 1 0 0 1 0-1.415Z"
            clipRule="evenodd"
        />
        <defs>
            <linearGradient
                id="object_details_header_svg__a"
                x1={12}
                x2={12}
                y1={0.5}
                y2={22.447}
                gradientUnits="userSpaceOnUse"
            >
                <stop stopColor="#6CFBD3" />
                <stop offset={1} stopColor="#75C5FC" />
            </linearGradient>
        </defs>
    </svg>
);
export default SvgObjectDetailsHeader;
