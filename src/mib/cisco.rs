/*
 * Copyright 2024 Oxide Computer Company
 */

use super::sublude::*;

pub fn populate(tree: &mut OidTree) -> Result<()> {
    add_from_instructions_under(
        tree,
        "enterprises",
        tree.oid_by_name("internet.private.enterprises")?.as_slice().to_vec(),
        &[
            ("cisco", "enterprises", 9),
            ("otherEnterprises", "cisco", 6),
            ("ciscoSB", "otherEnterprises", 1),
            ("switch001", "ciscoSB", 101),
            ("swInterfaces", "switch001", 43),
            ("swIfTable", "swInterfaces", 1),
            ("swIfEntry", "swIfTable", 1),
            ("swIfIndex", "swIfEntry", 1),
            ("swIfPhysAddressType", "swIfEntry", 2),
            ("swIfDuplexAdminMode", "swIfEntry", 3),
            ("swIfDuplexOperMode", "swIfEntry", 4),
            ("swIfBackPressureMode", "swIfEntry", 5),
            ("swIfTaggedMode", "swIfEntry", 6),
            ("swIfTransceiverType", "swIfEntry", 7),
            ("swIfLockAdminStatus", "swIfEntry", 8),
            ("swIfLockOperStatus", "swIfEntry", 9),
            ("swIfType", "swIfEntry", 10),
            ("swIfDefaultTag", "swIfEntry", 11),
            ("swIfDefaultPriority", "swIfEntry", 12),
            ("swIfAdminStatus", "swIfEntry", 13),
            ("swIfFlowControlMode", "swIfEntry", 14),
            ("swIfSpeedAdminMode", "swIfEntry", 15),
            ("swIfSpeedDuplexAutoNegotiation", "swIfEntry", 16),
            ("swIfOperFlowControlMode", "swIfEntry", 17),
            ("swIfOperSpeedDuplexAutoNegotiation", "swIfEntry", 18),
            ("swIfOperBackPressureMode", "swIfEntry", 19),
            ("swIfAdminLockAction", "swIfEntry", 20),
            ("swIfOperLockAction", "swIfEntry", 21),
            ("swIfAdminLockTrapEnable", "swIfEntry", 22),
            ("swIfOperLockTrapEnable", "swIfEntry", 23),
            ("swIfOperSuspendedStatus", "swIfEntry", 24),
            ("swIfLockOperTrapCount", "swIfEntry", 25),
            ("swIfLockAdminTrapFrequency", "swIfEntry", 26),
            ("swIfReActivate", "swIfEntry", 27),
            ("swIfAdminMdix", "swIfEntry", 28),
            ("swIfOperMdix", "swIfEntry", 29),
            ("swIfHostMode", "swIfEntry", 30),
            ("swIfSingleHostViolationAdminAction", "swIfEntry", 31),
            ("swIfSingleHostViolationOperAction", "swIfEntry", 32),
            ("swIfSingleHostViolationAdminTrapEnable", "swIfEntry", 33),
            ("swIfSingleHostViolationOperTrapEnable", "swIfEntry", 34),
            ("swIfSingleHostViolationOperTrapCount", "swIfEntry", 35),
            ("swIfSingleHostViolationAdminTrapFrequency", "swIfEntry", 36),
            ("swIfLockLimitationMode", "swIfEntry", 37),
            ("swIfLockMaxMacAddresses", "swIfEntry", 38),
            ("swIfLockMacAddressesCount", "swIfEntry", 39),
            (
                "swIfAdminSpeedDuplexAutoNegotiationLocalCapabilities",
                "swIfEntry",
                40,
            ),
            (
                "swIfOperSpeedDuplexAutoNegotiationLocalCapabilities",
                "swIfEntry",
                41,
            ),
            ("swIfSpeedDuplexNegotiationRemoteCapabilities", "swIfEntry", 42),
            ("swIfAdminComboMode", "swIfEntry", 43),
            ("swIfOperComboMode", "swIfEntry", 44),
            ("swIfAutoNegotiationMasterSlavePreference", "swIfEntry", 45),
            ("swIfPortCapabilities", "swIfEntry", 46),
            ("swIfPortStateDuration", "swIfEntry", 47),
            ("swIfApNegotiationLane", "swIfEntry", 48),
            ("swIfPortFecMode", "swIfEntry", 49),
            ("swIfPortNumOfLanes", "swIfEntry", 50),
        ],
    )
    .map_err(|e| anyhow!("cisco::populate: {e}"))?;

    Ok(())
}
