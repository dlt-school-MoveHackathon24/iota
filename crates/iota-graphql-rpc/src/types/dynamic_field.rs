// Copyright (c) Mysten Labs, Inc.
// Modifications Copyright (c) 2024 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use async_graphql::{
    connection::{Connection, CursorType, Edge},
    *,
};
use iota_indexer::{models::objects::StoredHistoryObject, types::OwnerType};
use iota_package_resolver::Resolver;
use iota_types::dynamic_field::{derive_dynamic_field_id, DynamicFieldInfo, DynamicFieldType};
use move_core_types::annotated_value::{self as A, MoveStruct};

use super::{
    base64::Base64,
    cursor::{Page, Target},
    iota_address::IotaAddress,
    move_object::MoveObject,
    move_value::MoveValue,
    object::{self, deserialize_move_struct, Object, ObjectKind, ObjectLookupKey},
    type_filter::ExactTypeFilter,
};
use crate::{
    consistency::{build_objects_query, consistent_range, View},
    context_data::package_cache::PackageCache,
    data::{Db, QueryExecutor},
    error::Error,
    filter,
    raw_query::RawQuery,
};

pub(crate) struct DynamicField {
    pub super_: MoveObject,
    pub df_object_id: IotaAddress,
    pub df_kind: DynamicFieldType,
}

#[derive(Union)]
pub(crate) enum DynamicFieldValue {
    MoveObject(MoveObject), // DynamicObject
    MoveValue(MoveValue),   // DynamicField
}

#[derive(InputObject)] // used as input object
pub(crate) struct DynamicFieldName {
    /// The string type of the DynamicField's 'name' field.
    /// A string representation of a Move primitive like 'u64', or a struct type
    /// like '0x2::kiosk::Listing'
    pub type_: ExactTypeFilter,
    /// The Base64 encoded bcs serialization of the DynamicField's 'name' field.
    pub bcs: Base64,
}

/// Dynamic fields are heterogeneous fields that can be added or removed at
/// runtime, and can have arbitrary user-assigned names. There are two sub-types
/// of dynamic fields:
///
/// 1) Dynamic Fields can store any value that has the `store` ability, however
///    an object stored in this kind of field will be considered wrapped and
///    will not be accessible directly via its ID by external tools (explorers,
///    wallets, etc) accessing storage.
/// 2) Dynamic Object Fields values must be Iota objects (have the `key` and
///    `store` abilities, and id: UID as the first field), but will still be
///    directly accessible off-chain via their object ID after being attached.
#[Object]
impl DynamicField {
    /// The string type, data, and serialized value of the DynamicField's 'name'
    /// field. This field is used to uniquely identify a child of the parent
    /// object.
    async fn name(&self, ctx: &Context<'_>) -> Result<Option<MoveValue>> {
        let resolver: &Resolver<PackageCache> = ctx
            .data()
            .map_err(|_| Error::Internal("Unable to fetch Package Cache.".to_string()))
            .extend()?;

        let (struct_tag, move_struct) = deserialize_move_struct(&self.super_.native, resolver)
            .await
            .extend()?;

        // Get TypeTag of the DynamicField name from StructTag of the MoveStruct
        let type_tag = DynamicFieldInfo::try_extract_field_name(&struct_tag, &self.df_kind)
            .map_err(|e| Error::Internal(e.to_string()))
            .extend()?;

        let name_move_value = extract_field_from_move_struct(move_struct, "name").extend()?;

        let undecorated = if self.df_kind == DynamicFieldType::DynamicObject {
            let inner_name_move_value = match name_move_value {
                A::MoveValue::Struct(inner_struct) => {
                    extract_field_from_move_struct(inner_struct, "name")
                }
                _ => Err(Error::Internal("Expected a wrapper struct".to_string())),
            }
            .extend()?;
            inner_name_move_value.undecorate()
        } else {
            name_move_value.undecorate()
        };

        let bcs = bcs::to_bytes(&undecorated)
            .map_err(|e| Error::Internal(format!("Failed to serialize object: {e}")))
            .extend()?;

        Ok(Some(MoveValue::new(type_tag, Base64::from(bcs))))
    }

    /// The actual data stored in the dynamic field.
    /// The returned dynamic field is an object if its return type is
    /// MoveObject, in which case it is also accessible off-chain via its
    /// address.
    async fn value(&self, ctx: &Context<'_>) -> Result<Option<DynamicFieldValue>> {
        if self.df_kind == DynamicFieldType::DynamicObject {
            // If `df_kind` is a DynamicObject, the object we are currently on is the field
            // object, and we must resolve one more level down to the value
            // object. Because we only have checkpoint-level granularity, we may
            // end up reading a later version of the value object. Thus, we use
            // the version of the field object to bound the value object at the
            // correct version.
            let obj = MoveObject::query(
                ctx.data_unchecked(),
                self.df_object_id,
                ObjectLookupKey::LatestAtParentVersion {
                    version: self.super_.super_.version_impl(),
                    checkpoint_viewed_at: self.super_.super_.checkpoint_viewed_at,
                },
            )
            .await
            .extend()?;
            Ok(obj.map(DynamicFieldValue::MoveObject))
        } else {
            let resolver: &Resolver<PackageCache> = ctx
                .data()
                .map_err(|_| Error::Internal("Unable to fetch Package Cache.".to_string()))
                .extend()?;

            let (struct_tag, move_struct) = deserialize_move_struct(&self.super_.native, resolver)
                .await
                .extend()?;

            // Get TypeTag of the DynamicField value from StructTag of the MoveStruct
            let type_tag = DynamicFieldInfo::try_extract_field_value(&struct_tag)
                .map_err(|e| Error::Internal(e.to_string()))
                .extend()?;

            let value_move_value = extract_field_from_move_struct(move_struct, "value").extend()?;

            let undecorated = value_move_value.undecorate();
            let bcs = bcs::to_bytes(&undecorated)
                .map_err(|e| Error::Internal(format!("Failed to serialize object: {e}")))
                .extend()?;

            Ok(Some(DynamicFieldValue::MoveValue(MoveValue::new(
                type_tag,
                Base64::from(bcs),
            ))))
        }
    }
}

impl DynamicField {
    /// Fetch a single dynamic field entry from the `db`, on `parent` object,
    /// with field name `name`, and kind `kind` (dynamic field or dynamic
    /// object field). The dynamic field is bound by the `parent_version` if
    /// provided - the fetched field will be the latest version at or before
    /// the provided version. If `parent_version` is not provided, the latest
    /// version of the field is returned as bounded by the
    /// `checkpoint_viewed_at` parameter.
    pub(crate) async fn query(
        db: &Db,
        parent: IotaAddress,
        parent_version: Option<u64>,
        name: DynamicFieldName,
        kind: DynamicFieldType,
        checkpoint_viewed_at: Option<u64>,
    ) -> Result<Option<DynamicField>, Error> {
        let type_ = match kind {
            DynamicFieldType::DynamicField => name.type_.0,
            DynamicFieldType::DynamicObject => {
                DynamicFieldInfo::dynamic_object_field_wrapper(name.type_.0).into()
            }
        };

        let field_id = derive_dynamic_field_id(parent, &type_, &name.bcs.0)
            .map_err(|e| Error::Internal(format!("Failed to derive dynamic field id: {e}")))?;

        use ObjectLookupKey as K;
        let key = match (parent_version, checkpoint_viewed_at) {
            (None, None) => K::Latest,
            (None, Some(checkpoint_viewed_at)) => K::LatestAt(checkpoint_viewed_at),
            (Some(version), checkpoint_viewed_at) => K::LatestAtParentVersion {
                version,
                checkpoint_viewed_at,
            },
        };

        let super_ = MoveObject::query(db, IotaAddress::from(field_id), key).await?;

        super_.map(Self::try_from).transpose()
    }

    /// Query the `db` for a `page` of dynamic fields attached to object with ID
    /// `parent`. The returned dynamic fields are bound by the
    /// `parent_version` if provided - each field will be the latest version
    /// at or before the provided version. If `parent_version` is not provided,
    /// the latest version of each field is returned as bounded by the
    /// `checkpoint_viewed-at` parameter.`
    pub(crate) async fn paginate(
        db: &Db,
        page: Page<object::Cursor>,
        parent: IotaAddress,
        parent_version: Option<u64>,
        checkpoint_viewed_at: Option<u64>,
    ) -> Result<Connection<String, DynamicField>, Error> {
        // If cursors are provided, defer to the `checkpoint_viewed_at` in the cursor if
        // they are consistent. Otherwise, use the value from the parameter, or
        // set to None. This is so that paginated queries are consistent with
        // the previous query that created the cursor.
        let cursor_viewed_at = page.validate_cursor_consistency()?;
        let checkpoint_viewed_at: Option<u64> = cursor_viewed_at.or(checkpoint_viewed_at);

        let Some(((prev, next, results), checkpoint_viewed_at)) = db
            .execute_repeatable(move |conn| {
                let Some((lhs, rhs)) = consistent_range(conn, checkpoint_viewed_at)? else {
                    return Ok::<_, diesel::result::Error>(None);
                };

                let result = page.paginate_raw_query::<StoredHistoryObject>(
                    conn,
                    rhs,
                    dynamic_fields_query(parent, parent_version, lhs as i64, rhs as i64, &page),
                )?;

                Ok(Some((result, rhs)))
            })
            .await?
        else {
            return Err(Error::Client(
                "Requested data is outside the available range".to_string(),
            ));
        };

        let mut conn: Connection<String, DynamicField> = Connection::new(prev, next);

        for stored in results {
            // To maintain consistency, the returned cursor should have the same upper-bound
            // as the checkpoint found on the cursor.
            let cursor = stored.cursor(checkpoint_viewed_at).encode_cursor();

            let object =
                Object::try_from_stored_history_object(stored, Some(checkpoint_viewed_at))?;

            let move_ = MoveObject::try_from(&object).map_err(|_| {
                Error::Internal(format!(
                    "Failed to deserialize as Move object: {}",
                    object.address
                ))
            })?;

            let dynamic_field = DynamicField::try_from(move_)?;

            conn.edges.push(Edge::new(cursor, dynamic_field));
        }

        Ok(conn)
    }
}

impl TryFrom<MoveObject> for DynamicField {
    type Error = Error;

    fn try_from(stored: MoveObject) -> Result<Self, Error> {
        let super_ = &stored.super_;

        let (df_object_id, df_kind) = match &super_.kind {
            ObjectKind::Live(_, stored) => stored
                .df_object_id
                .as_ref()
                .map(|id| (id, stored.df_kind))
                .ok_or_else(|| Error::Internal("Object is not a dynamic field.".to_string()))?,
            ObjectKind::Historical(_, stored) => stored
                .df_object_id
                .as_ref()
                .map(|id| (id, stored.df_kind))
                .ok_or_else(|| Error::Internal("Object is not a dynamic field.".to_string()))?,
            _ => {
                return Err(Error::Internal(
                    "A WrappedOrDeleted object cannot be converted into a DynamicField."
                        .to_string(),
                ));
            }
        };

        let df_object_id = IotaAddress::from_bytes(df_object_id).map_err(|e| {
            Error::Internal(format!("Failed to deserialize dynamic field ID: {e}."))
        })?;

        let df_kind = match df_kind {
            Some(0) => DynamicFieldType::DynamicField,
            Some(1) => DynamicFieldType::DynamicObject,
            Some(k) => {
                return Err(Error::Internal(format!(
                    "Unrecognized dynamic field kind: {k}."
                )));
            }
            None => return Err(Error::Internal("No dynamic field kind.".to_string())),
        };

        Ok(DynamicField {
            super_: stored,
            df_object_id,
            df_kind,
        })
    }
}

pub fn extract_field_from_move_struct(
    move_struct: MoveStruct,
    field_name: &str,
) -> Result<A::MoveValue, Error> {
    move_struct
        .fields
        .into_iter()
        .find_map(|(id, value)| {
            if id.to_string() == field_name {
                Some(value)
            } else {
                None
            }
        })
        .ok_or_else(|| Error::Internal(format!("Field '{}' not found", field_name)))
}

/// Builds the `RawQuery` for fetching dynamic fields attached to a parent
/// object. If `parent_version` is null, the latest version of each field within
/// the given checkpoint range [`lhs`, `rhs`] is returned, conditioned on the
/// fact that there is not a more recent version of the field.
///
/// If `parent_version` is provided, it is used to bound both the `candidates`
/// and `newer` objects subqueries. This is because the dynamic fields of a
/// parent at version v are dynamic fields owned by the parent whose versions
/// are <= v. Unlike object ownership, where owned and owner objects
/// can have arbitrary `object_version`s, dynamic fields on a parent cannot have
/// a version greater than its parent.
fn dynamic_fields_query(
    parent: IotaAddress,
    parent_version: Option<u64>,
    lhs: i64,
    rhs: i64,
    page: &Page<object::Cursor>,
) -> RawQuery {
    build_objects_query(
        View::Consistent,
        lhs,
        rhs,
        page,
        move |query| apply_filter(query, parent, parent_version),
        move |newer| {
            if let Some(parent_version) = parent_version {
                filter!(newer, format!("object_version <= {}", parent_version))
            } else {
                newer
            }
        },
    )
}

fn apply_filter(query: RawQuery, parent: IotaAddress, parent_version: Option<u64>) -> RawQuery {
    let query = filter!(
        query,
        format!(
            "owner_id = '\\x{}'::bytea AND owner_type = {} AND df_kind IS NOT NULL",
            hex::encode(parent.into_vec()),
            OwnerType::Object as i16
        )
    );

    if let Some(version) = parent_version {
        filter!(query, format!("object_version <= {}", version))
    } else {
        query
    }
}
