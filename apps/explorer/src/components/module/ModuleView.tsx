// Copyright (c) Mysten Labs, Inc.
// Modifications Copyright (c) 2024 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

import { type IotaMoveNormalizedType } from '@iota/iota-sdk/client';
import { normalizeIotaAddress } from '@iota/iota-sdk/utils';
import cl from 'clsx';
import { Highlight, Prism } from 'prism-react-renderer';
import 'prism-themes/themes/prism-one-light.css';
import { useMemo } from 'react';

import { useNormalizedMoveModule } from '~/hooks/useNormalizedMoveModule';
import { LinkWithQuery } from '~/components/ui';

import type { Language } from 'prism-react-renderer';

import styles from './ModuleView.module.css';

// Include Rust language support.
// TODO: Write a custom prismjs syntax for Move Bytecode.
globalThis.Prism = Prism;
// @ts-expect-error: This file is untyped:
import('prismjs/components/prism-rust').catch(() => {});

interface ModuleViewProps {
    id?: string;
    name: string;
    code: string;
}

interface TypeReference {
    address: string;
    module: string;
    name: string;
    typeArguments: IotaMoveNormalizedType[];
}

/** Takes a normalized move type and returns the address information contained within it */
function unwrapTypeReference(type: IotaMoveNormalizedType): null | TypeReference {
    if (typeof type === 'object') {
        if ('Struct' in type) {
            return type.Struct;
        }
        if ('Reference' in type) {
            return unwrapTypeReference(type.Reference);
        }
        if ('MutableReference' in type) {
            return unwrapTypeReference(type.MutableReference);
        }
        if ('Vector' in type) {
            return unwrapTypeReference(type.Vector);
        }
    }
    return null;
}

function ModuleView({ id, name, code }: ModuleViewProps): JSX.Element {
    const { data: normalizedModule } = useNormalizedMoveModule(id, name);
    const normalizedModuleReferences = useMemo(() => {
        const typeReferences: Record<string, TypeReference> = {};
        if (!normalizedModule) {
            return typeReferences;
        }
        Object.values(normalizedModule.exposedFunctions).forEach((exposedFunction) => {
            exposedFunction.parameters.forEach((param) => {
                const unwrappedType = unwrapTypeReference(param);
                if (!unwrappedType) return;
                typeReferences[unwrappedType.name] = unwrappedType;

                unwrappedType.typeArguments.forEach((typeArg) => {
                    const unwrappedTypeArg = unwrapTypeReference(typeArg);
                    if (!unwrappedTypeArg) return;
                    typeReferences[unwrappedTypeArg.name] = unwrappedTypeArg;
                });
            });
        });
        return typeReferences;
    }, [normalizedModule]);

    return (
        <section className={styles.modulewrapper}>
            <div className={cl(styles.code, styles.codeview)}>
                <Highlight code={code} language={'rust' as Language} theme={undefined}>
                    {({ className, style, tokens, getLineProps, getTokenProps }) => (
                        <pre className={className} style={style}>
                            {tokens.map((line, i) => (
                                <div
                                    {...getLineProps({ line, key: i })}
                                    key={i}
                                    className={styles.codeline}
                                >
                                    <div className={styles.codelinenumbers}>{i + 1}</div>

                                    {line.map((token, key) => {
                                        const reference =
                                            normalizedModuleReferences?.[token.content];

                                        if (
                                            (token.types.includes('class-name') ||
                                                token.types.includes('constant')) &&
                                            reference
                                        ) {
                                            const href = `/object/${reference.address}?module=${reference.module}`;

                                            return (
                                                <LinkWithQuery
                                                    key={key}
                                                    {...getTokenProps({
                                                        token,
                                                        key,
                                                    })}
                                                    to={href}
                                                    target={
                                                        normalizeIotaAddress(reference.address) ===
                                                        normalizeIotaAddress(id!)
                                                            ? undefined
                                                            : '_blank'
                                                    }
                                                />
                                            );
                                        }

                                        return (
                                            <span
                                                {...getTokenProps({
                                                    token,
                                                    key,
                                                })}
                                                key={key}
                                            />
                                        );
                                    })}
                                </div>
                            ))}
                        </pre>
                    )}
                </Highlight>
            </div>
        </section>
    );
}

export default ModuleView;
