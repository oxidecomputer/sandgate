/*
 * Copyright 2024 Oxide Computer Company
 */

use std::fmt::Display;

use crate::Oid;
use anyhow::{anyhow, bail, Result};

#[derive(Debug, Clone)]
pub struct OidTreeEntry {
    id: u64,
    value: u32,
    parent: Option<u64>,
    name: Option<String>,
    root: bool,
}

#[derive(Debug, Clone)]
pub struct OidTree {
    next_id: u64,
    nodes: Vec<OidTreeEntry>,
}

fn split_name(name: &str) -> Result<Vec<&str>> {
    if name.is_empty()
        || name
            .chars()
            .any(|c| !c.is_ascii_alphanumeric() && c != '.' && c != '-')
    {
        bail!("invalid OID name: {name:?}");
    }

    let t = name.split('.').collect::<Vec<_>>();
    if t.is_empty() {
        bail!("invalid OID name: {name:?}")
    }

    Ok(t)
}

pub struct OidName {
    components: Vec<String>,
}

impl OidName {
    pub fn basename(&self) -> &str {
        &self.components[self.components.len() - 1]
    }
}

impl Display for OidName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.components.join(".").fmt(f)
    }
}

impl Default for OidTree {
    fn default() -> Self {
        OidTree { next_id: 1000, nodes: Default::default() }
    }
}

impl OidTree {
    pub fn oid_by_name_under(&self, parent: Oid, name: &str) -> Result<Oid> {
        let t = split_name(name)?;

        /*
         * Find the parent OID in the tree:
         */
        let Ok(root) = self.find_oid(parent.as_slice()) else {
            bail!("no name found for parent OID {parent}");
        };

        /*
         * Now, walk down the tree we've been provided and match nodes.
         */
        let terminus = self
            .walk_down_under(root, &t)
            .map_err(|e| anyhow!("mapping OID {name:?} under {parent}: {e}"))?;

        Ok(self.oid_for_node(terminus))
    }

    pub fn oid_by_name(&self, name: &str) -> Result<Oid> {
        let t = split_name(name)?;

        /*
         * Find a root entry in the tree with this name:
         */
        let root = self
            .nodes
            .iter()
            .find(|n| n.root && n.name.as_deref() == Some(t[0]));
        let Some(root) = root else {
            bail!("could not find root node {:?}", t[0]);
        };

        /*
         * Now, walk down the tree we've been provided and match nodes.
         */
        let terminus = self
            .walk_down_under(root, &t[1..])
            .map_err(|e| anyhow!("mapping OID {name:?}: {e}"))?;

        Ok(self.oid_for_node(terminus))
    }

    /**
     * Walk down the tree from a given node, using each component of the
     * provided OID name to select the next node in the walk.
     */
    fn walk_down_under<'a>(
        &'a self,
        mut prior: &'a OidTreeEntry,
        names: &[&'a str],
    ) -> Result<&'a OidTreeEntry> {
        for &tt in names.iter() {
            let next = self.nodes.iter().find(|n| {
                !n.root
                    && n.name.as_deref() == Some(tt)
                    && n.parent == Some(prior.id)
            });

            if let Some(next) = next {
                prior = next;
            } else {
                bail!("could not find {tt:?}");
            }
        }

        Ok(prior)
    }

    /**
     * Walk from a node up to a root to generate a numeric oid.
     */
    fn oid_for_node<'a>(&'a self, mut prior: &'a OidTreeEntry) -> Oid {
        let mut out = Vec::new();
        loop {
            out.push(prior.value);
            if let Some(next) = prior.parent {
                prior = self.nodes.iter().find(|n| n.id == next).unwrap();
            } else {
                break;
            }
        }

        out.reverse();
        Oid(out.as_slice().try_into().unwrap())
    }

    pub fn oid_name(&self, oid: Oid) -> Result<OidName> {
        let oid = oid.as_slice();

        /*
         * Try to find an oid entry for this oid.
         */
        let mut n = oid.len();
        let mut out = Vec::new();
        let mut anchor = loop {
            if n == 0 {
                /*
                 * We give up.
                 */
                bail!("cannot do it");
            }

            if let Ok(ent) = self.find_oid(&oid[0..n]) {
                /*
                 * Found an anchor!
                 */
                break ent;
            } else {
                out.push(oid[n - 1].to_string());
                n -= 1;
            }
        };

        loop {
            if let Some(name) = anchor.name.as_deref() {
                out.push(name.to_string());
            } else {
                out.push(anchor.value.to_string());
            }

            if anchor.root {
                break;
            }

            if let Some(parent) = anchor.parent {
                anchor = self.nodes.iter().find(|n| n.id == parent).unwrap();
            } else {
                break;
            }
        }

        out.reverse();
        Ok(OidName { components: out })
    }

    fn find_oid(&self, oid: &[u32]) -> Result<&OidTreeEntry> {
        let mut prior = None;
        for &e in oid {
            let next =
                self.nodes.iter().find(|n| n.parent == prior && n.value == e);

            prior = Some(if let Some(next) = next {
                next.id
            } else {
                bail!("could not find oid {oid:?}");
            });
        }
        Ok(self.nodes.iter().find(|n| n.id == prior.unwrap()).unwrap())
    }

    fn find_oid_mut(&mut self, oid: &[u32]) -> Result<&mut OidTreeEntry> {
        let mut prior = None;
        for &e in oid {
            let next =
                self.nodes.iter().find(|n| n.parent == prior && n.value == e);

            prior = Some(if let Some(next) = next {
                next.id
            } else {
                bail!("could not find oid {oid:?}");
            });
        }
        Ok(self.nodes.iter_mut().find(|n| n.id == prior.unwrap()).unwrap())
    }

    pub fn add_oid_under(
        &mut self,
        parent: &[u32],
        oid: &[u32],
        name: &str,
    ) -> Result<Vec<u32>> {
        if oid.is_empty() || name.is_empty() {
            bail!("that wont work");
        }

        /*
         * Populate down the tree to the node we want to name.
         */
        let mut prior = Some(self.find_oid(parent)?.id);
        for &e in oid {
            let next = self
                .nodes
                .iter_mut()
                .find(|n| n.parent == prior && n.value == e);

            prior = Some(if let Some(next) = next {
                next.id
            } else {
                let id = self.next_id;
                self.next_id += 1;

                self.nodes.push(OidTreeEntry {
                    id,
                    value: e,
                    parent: prior,
                    name: None,
                    root: false,
                });
                id
            });
        }

        /*
         * Now that we're sure everything is there, locate the right node.
         */
        let mut full_oid = parent.to_vec();
        full_oid.extend(oid.to_vec());

        let ent = self.find_oid_mut(&full_oid).unwrap();
        ent.root = false;
        ent.name = Some(name.to_string());

        Ok(full_oid)
    }

    pub fn add_oid_root(
        &mut self,
        oid: &[u32],
        name: &str,
    ) -> Result<Vec<u32>> {
        if oid.is_empty() || name.is_empty() {
            bail!("that wont work");
        }

        /*
         * Populate down the tree to the node we want to name.
         */
        let mut prior = None;
        for &e in oid {
            let next = self
                .nodes
                .iter_mut()
                .find(|n| n.parent == prior && n.value == e);

            prior = Some(if let Some(next) = next {
                next.id
            } else {
                let id = self.next_id;
                self.next_id += 1;

                self.nodes.push(OidTreeEntry {
                    id,
                    value: e,
                    parent: prior,
                    name: None,
                    root: false,
                });
                id
            });
        }

        /*
         * Now that we're sure everything is there, locate the right node.
         */
        let ent = self.find_oid_mut(oid).unwrap();
        ent.root = true;
        ent.name = Some(name.to_string());

        Ok(oid.to_vec())
    }
}
