/*
 * Copyright 2024 Oxide Computer Company
 */

use std::collections::HashMap;

use crate::oidtree::OidTree;
use anyhow::{bail, Result};

pub mod apc;
pub mod cisco;
pub mod mib_2;

mod sublude {
    pub(crate) use super::add_from_instructions_under;
    pub(crate) use crate::oidtree::OidTree;
    pub(crate) use crate::walk::WalkedValues;
    pub(crate) use crate::{Client, Oid};
    #[allow(unused_imports)]
    pub(crate) use anyhow::{anyhow, bail, Result};
    pub(crate) use serde::Deserialize;
    pub(crate) use serde_repr::Deserialize_repr;
    pub(crate) use std::collections::{BTreeMap};
    pub(crate) use std::time::Duration;
}

pub fn base() -> OidTree {
    let mut tree = OidTree::default();

    /*
     * The path to the root is via: iso(1) org(3) dod(6) internet(1), but none
     * of that hierarchy is generally useful at this point so we'll start at
     * "internet" directly:
     */
    let internet =
        tree.add_oid_root(&[1, 3, 6, 1], "internet").expect("internet");

    /*
     * Base SNMPv2 definitions at the top of the tree:
     */
    add_from_instructions_under(
        &mut tree,
        "internet",
        internet,
        &[
            ("directory", "internet", 1),
            ("mgmt", "internet", 2),
            ("experimental", "internet", 3),
            ("private", "internet", 4),
            ("security", "internet", 5),
            ("snmpV2", "internet", 6),
            ("enterprises", "private", 1),
        ],
    )
    .expect("populate base");

    tree
}

pub(crate) fn add_from_instructions_under(
    tree: &mut OidTree,
    anchor_name: &str,
    anchor_oid: Vec<u32>,
    instructions: &[(&str, &str, u32)],
) -> Result<()> {
    let mut seen: HashMap<&str, Vec<u32>> = Default::default();
    seen.insert(anchor_name, anchor_oid);

    for (ins, under, rel) in instructions {
        if let Some(under) = seen.get(under).cloned() {
            let new = tree.add_oid_under(&under, &[*rel], ins)?;
            if seen.insert(ins, new).is_some() {
                bail!("adding: duplicate? {ins:?} -> {{ {under:?} {rel} }}");
            }
        } else {
            bail!("adding: could not find {{ {under:?} {rel} }} for {ins:?}");
        }
    }

    Ok(())
}
