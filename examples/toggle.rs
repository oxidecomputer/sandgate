/*
 * Copyright 2024 Oxide Computer Company
 */

use std::time::Duration;

use anyhow::{bail, Result};

use sandgate::mib::{self, apc::Pdu};
use sandgate::Client;

#[tokio::main]
async fn main() -> Result<()> {
    let opts = getopts::Options::new()
        .optopt("c", "", "community string", "COMMUNITY")
        .parsing_style(getopts::ParsingStyle::StopAtFirstFree)
        .parse(std::env::args_os().skip(1))?;

    if opts.free.len() != 3 {
        bail!("specify IP address of SNMP target, outlet number, on/off");
    }

    let outlet = opts.free[1].parse::<u32>()?;
    let cmd = match opts.free[2].as_str() {
        "on" => mib::apc::OutletCommand::ImmediateOn,
        "off" => mib::apc::OutletCommand::ImmediateOff,
        "reboot" => mib::apc::OutletCommand::ImmediateReboot,
        other => bail!("on or off, not {other:?}"),
    };

    let c = Client::builder()
        .community(opts.opt_str("c").as_deref().unwrap_or("public"))
        .with_oid_tree(|tree| mib::mib_2::populate(tree))?
        .with_oid_tree(|tree| mib::apc::populate(tree))?
        .build(opts.free[0].parse()?)
        .await?;

    let pdu: Pdu = Pdu::from_client(&c).await?;

    println!("pdu ident =          {:#?}", pdu.ident()?);
    println!();

    let cfg = pdu.outlet_config()?;
    let props = pdu.outlet_props()?;
    let status = pdu.outlet_status()?;
    let ctl = pdu.outlet_control()?;

    let Some((((cfg, props), status), ctl)) = cfg
        .get(&outlet)
        .zip(props.get(&outlet))
        .zip(status.get(&outlet))
        .zip(ctl.get(&outlet))
    else {
        bail!("could not get status for outlet {outlet}");
    };

    println!("pdu outlet config =  {:#?}", cfg);
    println!("pdu outlet props =   {:#?}", props);
    println!("pdu outlet status =  {:#?}", status);
    println!("pdu outlet control = {:#?}", ctl);
    println!();

    if status.is_command_pending()? {
        bail!("command already pending");
    }

    if ctl.command == cmd {
        bail!("command {cmd:?} already in effect");
    }

    println!("sending command {cmd:?} to outlet {outlet}...");
    let res = Pdu::send_command(&c, outlet, cmd).await?;
    println!("command result = {res:?}");

    loop {
        tokio::time::sleep(Duration::from_millis(250)).await;

        let (state, cmd, cmd_pending) = Pdu::poll_outlet(&c, outlet).await?;

        println!("state {state:?} cmd {cmd:?} cmd_pending {cmd_pending:?}");

        if matches!(cmd_pending, mib::apc::CommandPending::No) {
            break;
        }
    }

    Ok(())
}
