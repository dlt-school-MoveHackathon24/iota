// Copyright (c) Mysten Labs, Inc.
// Modifications Copyright (c) 2024 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0
import React from "react";
import clsx from "clsx";
import styles from "./styles.module.css";
export default function NavbarSearch({ children, className }) {
  return <div className={clsx(className, styles.searchBox)}>{children}</div>;
}
