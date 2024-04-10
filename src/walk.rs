/*
 * Copyright 2024 Oxide Computer Company
 */

use std::{
    collections::{BTreeMap, HashMap},
    ops::Bound,
    sync::Arc,
};

use crate::{value::Value, Oid};
use anyhow::{anyhow, bail, Result};
use serde::{de::value::MapDeserializer, Deserialize};

pub struct WalkedValues {
    pub(crate) values: BTreeMap<Oid, Value>,
    pub(crate) tree: Arc<crate::oidtree::OidTree>,
}

impl WalkedValues {
    pub fn extract_object<T>(
        &self,
        root: Oid,
        strip_name_prefix: &str,
    ) -> Result<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        let map = self
            .values
            .range(range_for_oid(root))
            .filter(|(oid, _)| {
                let rel =
                    oid.relative_to(root).expect("must be a child of oid");

                /*
                 * There may be tables or other objects underneath this one in
                 * the space, so skip over anything that is not a direct child
                 * value.
                 */
                rel.len() == 2 && rel.get(1).unwrap() == 0
            })
            .map(|(oid, val)| {
                let n = self.tree.oid_name(oid.parent().unwrap().into())?;
                let Some(n) = n.basename().strip_prefix(strip_name_prefix)
                else {
                    bail!("name {n} not prefixed with {strip_name_prefix:?}");
                };

                Ok((n.to_string(), val))
            })
            .collect::<Result<HashMap<String, &Value>>>()?;

        Ok(T::deserialize(MapDeserializer::new(map.into_iter()))?)
    }

    pub fn extract_table<T>(
        &self,
        table_size: Oid,
        table_entry: Oid,
        strip_name_prefix: &str,
    ) -> Result<BTreeMap<u32, T>>
    where
        T: for<'de> Deserialize<'de>,
    {
        /*
         * Get the size of the table from the results:
         */
        let Some(size) = self.values.get(&table_size.child(0).unwrap().into())
        else {
            bail!("could not locate table size at {table_size})");
        };

        let size: u32 = if let Some(i) = size.as_i32() {
            i.try_into()
                .map_err(|e| anyhow!("invalid size {i} at {table_size}: {e}"))?
        } else {
            bail!("invalid size {size:?} at {table_size}");
        };

        /*
         * Collect entries from the table:
         */
        let mut out: BTreeMap<u32, HashMap<String, &Value>> = BTreeMap::new();
        for (oid, val) in self.values.range(range_for_oid(table_entry)) {
            let rel =
                oid.relative_to(table_entry).expect("must be a child of oid");
            if rel.len() != 2 || rel.get(1).unwrap() == 0 {
                bail!("unusual table structure: {rel} under {oid}?");
            }

            let n = self.tree.oid_name(oid.parent().unwrap().into())?;
            let Some(n) = n.basename().strip_prefix(strip_name_prefix) else {
                bail!("name {n} not prefixed with {strip_name_prefix:?}");
            };

            let i = rel.get(1).unwrap();
            let map = out.entry(i).or_default();
            if map.insert(n.to_string(), val).is_some() {
                bail!("duplicate {n:?}[{i}] value?");
            }
        }

        for i in 1..=size {
            if !out.contains_key(&i) {
                bail!("table is missing index {i}?");
            }
        }

        /*
         * Deserialise the results!
         */
        out.into_iter()
            .map(|(idx, map)| {
                Ok((
                    idx,
                    T::deserialize(MapDeserializer::new(map.into_iter()))?,
                ))
            })
            .collect::<Result<_>>()
    }
}

/**
 * Generate a range that includes the provided oid, and all of its children, for
 * use with the BTreeMap range() walker.
 */
fn range_for_oid(oid: Oid) -> (Bound<Oid>, Bound<Oid>) {
    let last_id = *oid.as_slice().iter().last().unwrap();
    let one_after = oid
        .parent()
        .unwrap()
        .child(last_id.checked_add(1).unwrap())
        .unwrap()
        .into();
    (Bound::Included(oid), Bound::Excluded(one_after))
}
