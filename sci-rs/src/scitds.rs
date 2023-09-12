//! SCI Train Detection System

use crate::{
    impl_sci_message_type, impl_sci_messages_without_payload, ProtocolType, SCIMessageType,
    SCIPayload, SCITelegram,
};

// See Eu.Doc.44
impl_sci_message_type!(
    (scitds_fc, 0x0001),
    (scitds_update_filling_level, 0x0002),
    (scitds_drfc, 0x0003),
    (scitds_cancel, 0x0008),
    (scitds_command_rejected, 0x0006),
    (scitds_tvps_occupancy_status, 0x0007),
    (scitds_tvps_fc_p_failed, 0x0010),
    (scitds_tvps_fc_p_a_failed, 0x0011),
    (scitds_additional_information, 0x0012),
    (scitds_tdp_status, 0x000B)
);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum FCMode {
    U = 0x01,
    C = 0x02,
    PA = 0x03,
    P = 0x04,
    Ack = 0x05,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum OccupancyStatus {
    Vacant = 0x01,
    Occupied = 0x02,
    Disturbed = 0x03,
    WaitingForSweepingTrain = 0x04,
    WaitingForAck = 0x05,
    SweepingTrainDetected = 0x06,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum POMStatus {
    Ok = 0x01,
    NotOk = 0x02,
    NotApplicable = 0xFF,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum DisturbanceStatus {
    Operational = 0x01,
    Technical = 0x02,
    NotApplicable = 0xFF,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ChangeTrigger {
    PassingDetected = 0x01,
    CommandFromEILAccepted = 0x02,
    CommandFromMaintainerAccepted = 0x03,
    TechnicalFailure = 0x04,
    InitialSectionState = 0x05,
    InternalTrigger = 0x06,
    NotApplicable = 0xFF,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum RejectionReason {
    Operational = 0x01,
    Technical = 0x02,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum FCPFailureReason {
    IncorrectCountOfSweepingTrain = 0x01,
    Timeout = 0x02,
    IllegalBoundingDetectionPointConfig = 0x03,
    IntentionallyDeleted = 0x04,
    OutgoingAxleBeforeMinTimerExpiry = 0x05,
    ProcessCancelled = 0x06,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum StateOfPassing {
    NotPassed = 0x01,
    Passed = 0x02,
    Disturbed = 0x03,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum DirectionOfPassing {
    Reference = 0x01,
    AgainstReference = 0x02,
    WithoutIndicatedDirection = 0x03,
}

impl_sci_messages_without_payload!(
    ProtocolType::SCIProtocolTDS,
    (
        (
            update_filling_level,
            SCIMessageType::scitds_update_filling_level()
        ),
        (cancel, SCIMessageType::scitds_cancel()),
        (drfc, SCIMessageType::scitds_drfc())
    )
);

impl SCITelegram {
    pub fn fc(sender: &str, receiver: &str, mode: FCMode) -> Self {
        Self {
            protocol_type: ProtocolType::SCIProtocolTDS,
            message_type: SCIMessageType::scitds_fc(),
            sender: sender.to_string(),
            receiver: receiver.to_string(),
            payload: SCIPayload::from_slice(&[mode as u8]),
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn tvps_occupancy_status(
        sender: &str,
        receiver: &str,
        occupancy_status: OccupancyStatus,
        can_be_forced_to_clear: bool,
        filling_level: i16,
        pom_status: POMStatus,
        disturbance_status: DisturbanceStatus,
        change_trigger: ChangeTrigger,
    ) -> Self {
        let filling_level_bytes = filling_level.to_be_bytes();
        Self {
            protocol_type: ProtocolType::SCIProtocolTDS,
            message_type: SCIMessageType::scitds_tvps_occupancy_status(),
            sender: sender.to_string(),
            receiver: receiver.to_string(),
            payload: SCIPayload::from_slice(&[
                occupancy_status as u8,
                match can_be_forced_to_clear {
                    true => 0x01,
                    false => 0x02,
                },
                filling_level_bytes[0],
                filling_level_bytes[1],
                pom_status as u8,
                disturbance_status as u8,
                change_trigger as u8,
            ]),
        }
    }

    pub fn command_rejected(sender: &str, receiver: &str, reason: RejectionReason) -> Self {
        Self {
            protocol_type: ProtocolType::SCIProtocolTDS,
            message_type: SCIMessageType::scitds_command_rejected(),
            sender: sender.to_string(),
            receiver: receiver.to_string(),
            payload: SCIPayload::from_slice(&[reason as u8]),
        }
    }

    pub fn tvps_fc_p_failed(sender: &str, receiver: &str, reason: FCPFailureReason) -> Self {
        Self {
            protocol_type: ProtocolType::SCIProtocolTDS,
            message_type: SCIMessageType::scitds_tvps_fc_p_failed(),
            sender: sender.to_string(),
            receiver: receiver.to_string(),
            payload: SCIPayload::from_slice(&[reason as u8]),
        }
    }

    pub fn tvps_fc_p_a_failed(sender: &str, receiver: &str, reason: FCPFailureReason) -> Self {
        Self {
            protocol_type: ProtocolType::SCIProtocolTDS,
            message_type: SCIMessageType::scitds_tvps_fc_p_a_failed(),
            sender: sender.to_string(),
            receiver: receiver.to_string(),
            payload: SCIPayload::from_slice(&[reason as u8]),
        }
    }

    /// Speed and wheel diameter are encoded as BCD.
    /// Pass them as an array of u8 digits.
    pub fn additional_information(
        sender: &str,
        receiver: &str,
        speed: [u8; 4],
        wheel_diameter: [u8; 4],
    ) -> Self {
        let speed_bcd = to_bcd(speed).to_be_bytes();
        let wheel_diameter_bcd = to_bcd(wheel_diameter).to_be_bytes();
        Self {
            protocol_type: ProtocolType::SCIProtocolTDS,
            message_type: SCIMessageType::scitds_additional_information(),
            sender: sender.to_string(),
            receiver: receiver.to_string(),
            payload: SCIPayload::from_slice(&[
                speed_bcd[0],
                speed_bcd[1],
                wheel_diameter_bcd[0],
                wheel_diameter_bcd[1],
            ]),
        }
    }

    pub fn tdp_status(
        sender: &str,
        receiver: &str,
        state_of_passing: StateOfPassing,
        direction_of_passing: DirectionOfPassing,
    ) -> Self {
        Self {
            protocol_type: ProtocolType::SCIProtocolTDS,
            message_type: SCIMessageType::scitds_tdp_status(),
            sender: sender.to_string(),
            receiver: receiver.to_string(),
            payload: SCIPayload::from_slice(&[state_of_passing as u8, direction_of_passing as u8]),
        }
    }
}

fn to_bcd(digits: [u8; 4]) -> u16 {
    assert!(
        digits.iter().all(|&d| d <= 9),
        "BCD Digits must be between 0 and 9"
    );
    let digit_0 = (digits[0] << 4) + digits[1];
    let digit_1 = (digits[2] << 4) + digits[3];
    u16::from_be_bytes([digit_0, digit_1])
}

#[cfg(test)]
mod tests {
    use crate::scitds::to_bcd;

    #[test]
    fn test_bcd() {
        assert_eq!(to_bcd([0, 0, 0, 1]), 1);
        assert_eq!(to_bcd([0, 0, 1, 1]), 17);
        assert_eq!(to_bcd([0, 1, 1, 1]), 273);
        assert_eq!(to_bcd([1, 1, 1, 1]), 4369);
    }
}
