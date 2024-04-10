/*
 * Copyright 2024 Oxide Computer Company
 */

use std::{
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr},
    ops::Deref,
    result::Result as SResult,
    sync::Arc,
    time::Duration,
};

use anyhow::{anyhow, Result};

/*
 * Re-export the csnmp module we're using:
 */
pub use csnmp;
use csnmp::ObjectIdentifier;
use serde::{de::Visitor, Deserialize, Deserializer};

pub mod mib;
pub mod oidtree;
pub mod value;
pub mod walk;

#[derive(Clone, Copy, Hash, Eq, PartialEq, PartialOrd, Ord)]
pub struct Oid(ObjectIdentifier);

impl std::fmt::Debug for Oid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        format_args!("Oid({})", self).fmt(f)
    }
}

impl<'de> Deserialize<'de> for Oid {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(OidVisitor)
    }
}

struct OidVisitor;

impl<'de> Visitor<'de> for OidVisitor {
    type Value = Oid;

    fn expecting(
        &self,
        formatter: &mut std::fmt::Formatter,
    ) -> std::fmt::Result {
        formatter.write_str("an OID (a sequence of u32)")
    }

    fn visit_seq<A>(
        self,
        mut seq: A,
    ) -> std::result::Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        let mut nums: Vec<u32> =
            Vec::with_capacity(seq.size_hint().unwrap_or(64));

        while let Some(val) = seq.next_element()? {
            nums.push(val);
        }

        Ok(Oid(ObjectIdentifier::try_from(nums.as_slice()).map_err(|e| {
            serde::de::Error::custom(format!("invalid OID: {e}"))
        })?))
    }
}

#[derive(Clone, Copy, Hash, Eq, PartialEq, PartialOrd, Ord, Debug)]
pub struct RelativeOid(ObjectIdentifier);

impl Deref for Oid {
    type Target = ObjectIdentifier;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Deref for RelativeOid {
    type Target = ObjectIdentifier;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::fmt::Display for Oid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl std::fmt::Display for RelativeOid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl Oid {
    fn relative_to(&self, base: Oid) -> Option<RelativeOid> {
        self.0.relative_to(&base.0).map(|oid| oid.into())
    }
}

impl From<ObjectIdentifier> for Oid {
    fn from(oid: ObjectIdentifier) -> Self {
        Oid(oid)
    }
}

impl From<ObjectIdentifier> for RelativeOid {
    fn from(oid: ObjectIdentifier) -> Self {
        RelativeOid(oid)
    }
}

pub struct Client {
    snmp: csnmp::Snmp2cClient,
    tree: Arc<oidtree::OidTree>,
}

impl Client {
    pub fn builder() -> ClientBuilder {
        ClientBuilder {
            bind_address: None,
            target_port: 161,
            community: b"public".to_vec(),
            timeout: Duration::from_secs(5),
            retries: 0,
            tree: mib::base(),
        }
    }

    pub async fn set(
        &self,
        oid: Oid,
        value: value::Value,
    ) -> SResult<value::Value, csnmp::SnmpClientError> {
        self.snmp.set(oid.0, value.0).await.map(|val| value::Value(val))
    }

    pub async fn walk(&self, top: Oid) -> Result<walk::WalkedValues> {
        let res = self.snmp.walk_bulk(top.0, 63).await?;

        Ok(walk::WalkedValues {
            values: res
                .into_iter()
                .map(|(k, v)| (Oid(k), value::Value(v)))
                .collect(),
            tree: Arc::clone(&self.tree),
        })
    }

    pub fn tree(&self) -> &oidtree::OidTree {
        &self.tree
    }
}

pub struct ClientBuilder {
    bind_address: Option<SocketAddr>,
    target_port: u16,
    community: Vec<u8>,
    timeout: Duration,
    retries: usize,
    tree: oidtree::OidTree,
}

impl ClientBuilder {
    pub fn port(&mut self, port: u16) -> &mut Self {
        self.target_port = port;
        self
    }

    pub fn community<C: AsRef<[u8]>>(&mut self, community: C) -> &mut Self {
        self.community = community.as_ref().to_vec();
        self
    }

    pub fn with_oid_tree<E: std::fmt::Display + Send + Sync>(
        &mut self,
        func: impl Fn(&mut oidtree::OidTree) -> std::result::Result<(), E>,
    ) -> Result<&mut Self> {
        func(&mut self.tree)
            .map_err(|e| anyhow!("client builder with_oid_tree(): {e}"))?;
        Ok(self)
    }

    pub async fn build(&self, target_ip: IpAddr) -> Result<Client> {
        /*
         * Generate the socket address for the target:
         */
        let target = SocketAddr::new(target_ip, self.target_port);

        /*
         * Pick a local bind address based on the address family if one was not
         * provided:
         */
        let bind = if let Some(ba) = self.bind_address {
            ba
        } else {
            SocketAddr::new(
                match target {
                    SocketAddr::V4(_) => Ipv4Addr::UNSPECIFIED.into(),
                    SocketAddr::V6(_) => Ipv6Addr::UNSPECIFIED.into(),
                },
                0,
            )
        };

        let snmp = csnmp::Snmp2cClient::new(
            target,
            self.community.clone(),
            Some(bind),
            Some(self.timeout),
            self.retries,
        )
        .await?;

        Ok(Client { snmp, tree: Arc::new(self.tree.clone()) })
    }
}
