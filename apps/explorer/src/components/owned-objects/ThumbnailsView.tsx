// Copyright (c) Mysten Labs, Inc.
// Modifications Copyright (c) 2024 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

import { type IotaObjectResponse } from '@iota/iota-sdk/client';
import { formatAddress } from '@iota/iota-sdk/utils';
import { Placeholder, Text } from '@iota/ui';

import { ObjectLink, ObjectVideoImage } from '~/components/ui';
import { useResolveVideo } from '~/hooks/useResolveVideo';
import { parseObjectType, trimStdLibPrefix } from '~/lib/utils';

function Thumbnail({ obj }: { obj: IotaObjectResponse }): JSX.Element {
    const video = useResolveVideo(obj);
    const displayMeta = obj.data?.display?.data;
    const src = displayMeta?.image_url || '';
    const name = displayMeta?.name ?? displayMeta?.description;
    const type = trimStdLibPrefix(parseObjectType(obj));
    const id = obj.data?.objectId;
    const displayName = name || formatAddress(id!);

    return (
        <div>
            <ObjectLink
                display="flex"
                objectId={id!}
                label={
                    <div className="group relative">
                        <ObjectVideoImage
                            fadeIn
                            disablePreview
                            title={name || '--'}
                            subtitle={type}
                            src={src}
                            video={video}
                            variant="medium"
                        />
                        <div className="absolute bottom-2 left-1/2 hidden w-10/12 -translate-x-1/2 justify-center rounded-lg bg-white/80 px-2 py-1 backdrop-blur group-hover:flex">
                            <Text variant="subtitle/medium" color="steel-dark" truncate>
                                {displayName}
                            </Text>
                        </div>
                    </div>
                }
            />
        </div>
    );
}

function ThumbnailsOnlyLoading({ limit }: { limit: number }): JSX.Element {
    return (
        <>
            {new Array(limit).fill(0).map((_, index) => (
                <div key={index} className="h-16 w-16 md:h-31.5 md:w-31.5">
                    <Placeholder rounded="lg" height="100%" />
                </div>
            ))}
        </>
    );
}

interface ThumbnailsViewViewProps {
    limit: number;
    data?: IotaObjectResponse[];
    loading?: boolean;
}

export function ThumbnailsView({ data, loading, limit }: ThumbnailsViewViewProps): JSX.Element {
    return (
        <div className="flex flex-row flex-wrap gap-2 overflow-auto md:gap-4">
            {loading ? (
                <ThumbnailsOnlyLoading limit={limit} />
            ) : (
                data?.map((obj) => <Thumbnail key={obj.data?.objectId} obj={obj} />)
            )}
        </div>
    );
}
