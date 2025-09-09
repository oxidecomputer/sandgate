/*
 * Copyright 2025 Oxide Computer Company
 */

use anyhow::{bail, Result};

use sandgate::mib::{
    self,
    apc::{Pdu, Pdu1},
    mib_2::System,
};
use sandgate::Client;

#[tokio::main]
async fn main() -> Result<()> {
    let opts = getopts::Options::new()
        .optopt("c", "", "community string", "COMMUNITY")
        .parsing_style(getopts::ParsingStyle::StopAtFirstFree)
        .parse(std::env::args_os().skip(1))?;

    if opts.free.len() != 1 {
        bail!("specify IP address of SNMP target");
    }

    let c = Client::builder()
        .community(opts.opt_str("c").as_deref().unwrap_or("public"))
        .with_oid_tree(|tree| mib::mib_2::populate(tree))?
        .with_oid_tree(|tree| mib::apc::populate(tree))?
        .build(opts.free[0].parse()?)
        .await?;

    let s: System = System::from_client(&c).await?;
    println!("system = {s:#?}");
    println!("vendor OID is {}", s.object_id());
    println!(
        "   = {}",
        c.tree()
            .oid_name(s.object_id())
            .map(|s| s.to_string())
            .ok()
            .as_deref()
            .unwrap_or("?")
    );

    if let Err(e) = inspect_pdu2(&c).await {
        println!("device does not support rPDU2 ({e}), trying rPDU...");
        inspect_pdu1(&c).await?;
    }

    Ok(())
}

async fn inspect_pdu2(c: &Client) -> Result<()> {
    let pdu: Pdu = Pdu::from_client(c).await?;
    println!("pdu ident =          {:#?}", pdu.ident()?);
    println!();

    println!("pdu bank config =    {:#?}", pdu.bank_config()?);
    println!("pdu bank props =     {:#?}", pdu.bank_props()?);
    println!("pdu bank status =    {:#?}", pdu.bank_status()?);
    println!();

    println!("pdu outlet config =  {:#?}", pdu.outlet_config()?);
    println!("pdu outlet props =   {:#?}", pdu.outlet_props()?);
    println!("pdu outlet status =  {:#?}", pdu.outlet_status()?);
    println!("pdu outlet control = {:#?}", pdu.outlet_control()?);
    println!();

    Ok(())
}

async fn inspect_pdu1(c: &Client) -> Result<()> {
    let pdu: Pdu1 = Pdu1::from_client(c).await?;
    println!("pdu ident =          {:#?}", pdu.ident()?);
    println!();

    println!("pdu outlet config =  {:#?}", pdu.outlet_config()?);
    println!("pdu outlet status =  {:#?}", pdu.outlet_status()?);
    println!("pdu outlet control = {:#?}", pdu.outlet_control()?);
    println!();

    Ok(())
}
