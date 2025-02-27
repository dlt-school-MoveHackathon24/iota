// Copyright (c) 2024 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

import { ButtonHtmlType } from '../button.enums';
import { ButtonVariantProps } from './button-variants.types';
import cx from 'classnames';

export function ButtonUnstyled({
    htmlType = ButtonHtmlType.Button,
    children,
    className,
    tabIndex = 0,
    ...buttonProps
}: ButtonVariantProps): React.JSX.Element {
    return (
        <button
            type={htmlType}
            {...buttonProps}
            className={cx('appearance-none', className)}
            tabIndex={tabIndex}
        >
            {children}
        </button>
    );
}
