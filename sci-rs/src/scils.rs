//! # SCI Light Signal
//!
//! The Standard Communication Interface for light signals.

#[derive(Debug, Clone, Copy)]
pub enum SciLsError {
    InvalidMainSignalAspect(u8),
    InvalidAdditionalSignalAspect(u8),
    InvalidZs2Aspect(u8),
    InvalidZs3Aspect(u8),
    InvalidDepreciationInformation(u8),
    InvalidDrivewayInformation(u8),
    InvalidDarkSwitching(u8),
    InvalidBrightness(u8),
}

use crate::SciError;

use super::{ProtocolType, SCIMessageType, SCIPayload, SCITelegram};

impl SCIMessageType {
    pub const fn scils_show_signal_aspect() -> Self {
        Self(0x0001)
    }

    pub const fn scils_change_brightness() -> Self {
        Self(0x0002)
    }

    pub const fn scils_signal_aspect_status() -> Self {
        Self(0x0003)
    }

    pub const fn scils_brightness_status() -> Self {
        Self(0x0004)
    }
}

/// The possible aspects of a main signal
#[derive(Default, Clone, Copy, PartialEq, Debug)]
#[repr(u8)]
pub enum SCILSMain {
    Hp0 = 0x01,
    Hp0PlusSh1 = 0x02,
    Hp0WithDrivingIndicator = 0x03,
    Ks1 = 0x04,
    Ks1Flashing = 0x05,
    Ks1FlashingWithAdditionalLight = 0x06,
    Ks2 = 0x07,
    Ks2WithAdditionalLight = 0x08,
    Sh1 = 0x09,
    IdLight = 0x0A,
    Hp0Hv = 0xA0,
    Hp1 = 0xA1,
    Hp2 = 0xA2,
    Vr0 = 0xB0,
    Vr1 = 0xB1,
    Vr2 = 0xB2,
    #[default]
    Off = 0xFF,
}

impl TryFrom<u8> for SCILSMain {
    type Error = SciError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x01 => Ok(Self::Hp0),
            0x02 => Ok(Self::Hp0PlusSh1),
            0x03 => Ok(Self::Hp0WithDrivingIndicator),
            0x04 => Ok(Self::Ks1),
            0x05 => Ok(Self::Ks1Flashing),
            0x06 => Ok(Self::Ks1FlashingWithAdditionalLight),
            0x07 => Ok(Self::Ks2),
            0x08 => Ok(Self::Ks2WithAdditionalLight),
            0x09 => Ok(Self::Sh1),
            0x0A => Ok(Self::IdLight),
            0xA0 => Ok(Self::Hp0Hv),
            0xA1 => Ok(Self::Hp1),
            0xA2 => Ok(Self::Hp2),
            0xB0 => Ok(Self::Vr0),
            0xB1 => Ok(Self::Vr1),
            0xB2 => Ok(Self::Vr2),
            0xFF => Ok(Self::Off),
            v => Err(SciLsError::InvalidMainSignalAspect(v).into()),
        }
    }
}

/// The possible types of an additional signal
/// (excluding Zs2(v) and Zs3(v) which can show
/// additional information and are listed separately)
#[derive(Default, Clone, Copy)]
#[repr(u8)]
pub enum SCILSAdditional {
    Zs1 = 0x01,
    Zs7 = 0x02,
    Zs8 = 0x03,
    Zs6 = 0x04,
    Zs13 = 0x05,
    #[default]
    Off = 0xFF,
}

impl TryFrom<u8> for SCILSAdditional {
    type Error = SciError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x01 => Ok(Self::Zs1),
            0x02 => Ok(Self::Zs7),
            0x03 => Ok(Self::Zs8),
            0x04 => Ok(Self::Zs6),
            0x05 => Ok(Self::Zs13),
            0xFF => Ok(Self::Off),
            v => Err(SciLsError::InvalidAdditionalSignalAspect(v).into()),
        }
    }
}

/// Possible aspects for Zs3 and Zs3v signals
#[derive(Default, Clone, Copy)]
#[repr(u8)]
pub enum SCILSZs3 {
    Index1 = 0x01,
    Index2 = 0x02,
    Index3 = 0x03,
    Index4 = 0x04,
    Index5 = 0x05,
    Index6 = 0x06,
    Index7 = 0x07,
    Index8 = 0x08,
    Index9 = 0x09,
    Index10 = 0x0A,
    Index11 = 0x0B,
    Index12 = 0x0C,
    Index13 = 0x0D,
    Index14 = 0x0E,
    Index15 = 0x0F,
    #[default]
    Off = 0xFF,
}

impl TryFrom<u8> for SCILSZs3 {
    type Error = SciError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x01 => Ok(Self::Index1),
            0x02 => Ok(Self::Index2),
            0x03 => Ok(Self::Index3),
            0x04 => Ok(Self::Index4),
            0x05 => Ok(Self::Index5),
            0x06 => Ok(Self::Index6),
            0x07 => Ok(Self::Index7),
            0x08 => Ok(Self::Index8),
            0x09 => Ok(Self::Index9),
            0x0A => Ok(Self::Index10),
            0x0B => Ok(Self::Index11),
            0x0C => Ok(Self::Index12),
            0x0D => Ok(Self::Index13),
            0x0E => Ok(Self::Index14),
            0x0F => Ok(Self::Index15),
            0xFF => Ok(Self::Off),
            v => Err(SciLsError::InvalidZs3Aspect(v).into()),
        }
    }
}

/// Possible aspects for Zs2 and Zs2v signals
#[derive(Default, Clone, Copy)]
#[repr(u8)]
pub enum SCILSZs2 {
    LetterA = 0x01,
    LetterB = 0x02,
    LetterC = 0x03,
    LetterD = 0x04,
    LetterE = 0x05,
    LetterF = 0x06,
    LetterG = 0x07,
    LetterH = 0x08,
    LetterI = 0x09,
    LetterJ = 0x0A,
    LetterK = 0x0B,
    LetterL = 0x0C,
    LetterM = 0x0D,
    LetterN = 0x0E,
    LetterO = 0x0F,
    LetterP = 0x10,
    LetterQ = 0x11,
    LetterR = 0x12,
    LetterS = 0x13,
    LetterT = 0x14,
    LetterU = 0x15,
    LetterV = 0x16,
    LetterW = 0x17,
    LetterX = 0x18,
    LetterY = 0x19,
    LetterZ = 0x1A,
    #[default]
    Off = 0xFF,
}

impl TryFrom<u8> for SCILSZs2 {
    type Error = SciError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x01 => Ok(Self::LetterA),
            0x02 => Ok(Self::LetterB),
            0x03 => Ok(Self::LetterC),
            0x04 => Ok(Self::LetterD),
            0x05 => Ok(Self::LetterE),
            0x06 => Ok(Self::LetterF),
            0x07 => Ok(Self::LetterG),
            0x08 => Ok(Self::LetterH),
            0x09 => Ok(Self::LetterI),
            0x0A => Ok(Self::LetterJ),
            0x0B => Ok(Self::LetterK),
            0x0C => Ok(Self::LetterL),
            0x0D => Ok(Self::LetterM),
            0x0E => Ok(Self::LetterN),
            0x0F => Ok(Self::LetterO),
            0x10 => Ok(Self::LetterP),
            0x11 => Ok(Self::LetterQ),
            0x12 => Ok(Self::LetterR),
            0x13 => Ok(Self::LetterS),
            0x14 => Ok(Self::LetterT),
            0x15 => Ok(Self::LetterU),
            0x16 => Ok(Self::LetterV),
            0x17 => Ok(Self::LetterW),
            0x18 => Ok(Self::LetterX),
            0x19 => Ok(Self::LetterY),
            0x1A => Ok(Self::LetterZ),
            0xFF => Ok(Self::Off),
            v => Err(SciLsError::InvalidZs2Aspect(v).into()),
        }
    }
}

#[derive(Default, Clone, Copy)]
#[repr(u8)]
pub enum SCILSDepreciationInformation {
    Type1 = 0x01,
    Type2 = 0x02,
    Type3 = 0x03,
    #[default]
    NoInformation = 0xFF,
}

impl TryFrom<u8> for SCILSDepreciationInformation {
    type Error = SciError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x01 => Ok(Self::Type1),
            0x02 => Ok(Self::Type2),
            0x03 => Ok(Self::Type3),
            0xFF => Ok(Self::NoInformation),
            v => Err(SciLsError::InvalidDepreciationInformation(v).into()),
        }
    }
}

#[derive(Default, Clone, Copy)]
#[repr(u8)]
pub enum SCILSDrivewayInformation {
    Way1 = 0x1,
    Way2 = 0x2,
    Way3 = 0x3,
    Way4 = 0x4,
    #[default]
    NoInformation = 0xF,
}

impl TryFrom<u8> for SCILSDrivewayInformation {
    type Error = SciError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x1 => Ok(Self::Way1),
            0x2 => Ok(Self::Way2),
            0x3 => Ok(Self::Way3),
            0x4 => Ok(Self::Way4),
            0xFF => Ok(Self::NoInformation),
            v => Err(SciLsError::InvalidDrivewayInformation(v).into()),
        }
    }
}

#[derive(Default, Clone, Copy)]
#[repr(u8)]
pub enum SCILSDarkSwitching {
    #[default]
    Show = 0x01,
    Dark = 0xFF,
}

impl TryFrom<u8> for SCILSDarkSwitching {
    type Error = SciError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x01 => Ok(Self::Show),
            0xFF => Ok(Self::Dark),
            v => Err(SciLsError::InvalidDarkSwitching(v).into()),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
#[repr(u8)]
pub enum SCILSBrightness {
    Day = 0x01,
    Night = 0x02,
    Undefined = 0xFF, // Only allowed in telegram: Message Configured Luminosity
}

impl TryFrom<u8> for SCILSBrightness {
    type Error = SciError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x01 => Ok(Self::Day),
            0x02 => Ok(Self::Night),
            0xFF => Ok(Self::Undefined),
            v => Err(SciLsError::InvalidBrightness(v).into()),
        }
    }
}

#[derive(Clone)]
/// A complete signal aspect.
pub struct SCILSSignalAspect {
    main: SCILSMain,
    additional: SCILSAdditional,
    zs3: SCILSZs3,
    zs3v: SCILSZs3,
    zs2: SCILSZs2,
    zs2v: SCILSZs2,
    depreciation_information: SCILSDepreciationInformation,
    upstream_driveway_information: SCILSDrivewayInformation,
    downstream_driveway_information: SCILSDrivewayInformation,
    dark_switching: SCILSDarkSwitching,
    nationally_specified_information: [u8; 9],
}

impl SCILSSignalAspect {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        main: SCILSMain,
        additional: SCILSAdditional,
        zs3: SCILSZs3,
        zs3v: SCILSZs3,
        zs2: SCILSZs2,
        zs2v: SCILSZs2,
        depreciation_information: SCILSDepreciationInformation,
        upstream_driveway_information: SCILSDrivewayInformation,
        downstream_driveway_information: SCILSDrivewayInformation,
        dark_switching: SCILSDarkSwitching,
        nationally_specified_information: [u8; 9],
    ) -> Self {
        Self {
            main,
            additional,
            zs3,
            zs3v,
            zs2,
            zs2v,
            depreciation_information,
            upstream_driveway_information,
            downstream_driveway_information,
            dark_switching,
            nationally_specified_information,
        }
    }

    pub fn main(&self) -> SCILSMain {
        self.main
    }

    pub fn additional(&self) -> SCILSAdditional {
        self.additional
    }

    pub fn zs3(&self) -> SCILSZs3 {
        self.zs3
    }

    pub fn zs3v(&self) -> SCILSZs3 {
        self.zs3v
    }

    pub fn zs2(&self) -> SCILSZs2 {
        self.zs2
    }

    pub fn zs2v(&self) -> SCILSZs2 {
        self.zs2v
    }

    pub fn depreciation_information(&self) -> SCILSDepreciationInformation {
        self.depreciation_information
    }

    pub fn upstream_driveway_information(&self) -> SCILSDrivewayInformation {
        self.upstream_driveway_information
    }

    pub fn downstream_driveway_information(&self) -> SCILSDrivewayInformation {
        self.downstream_driveway_information
    }

    pub fn dark_switching(&self) -> SCILSDarkSwitching {
        self.dark_switching
    }

    pub fn nationally_specified_information(&self) -> &[u8] {
        &self.nationally_specified_information
    }
}

impl From<SCILSSignalAspect> for SCIPayload {
    fn from(value: SCILSSignalAspect) -> Self {
        let mut data = vec![0; 9];
        data[0] = value.main as u8;
        data[1] = value.additional as u8;
        data[2] = value.zs3 as u8;
        data[3] = value.zs3v as u8;
        data[4] = value.zs2 as u8;
        data[5] = value.zs2v as u8;
        data[6] = value.depreciation_information as u8;
        let mut driveway_info = (value.downstream_driveway_information as u8) << 4;
        driveway_info |= value.upstream_driveway_information as u8;
        data[7] = driveway_info;
        data[8] = value.dark_switching as u8;

        Self::from_slice(&data)
    }
}

impl TryFrom<&[u8]> for SCILSSignalAspect {
    type Error = SciError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let main = SCILSMain::try_from(value[0])?;
        let additional = SCILSAdditional::try_from(value[1])?;
        let zs3 = SCILSZs3::try_from(value[2])?;
        let zs3v = SCILSZs3::try_from(value[3])?;
        let zs2 = SCILSZs2::try_from(value[4])?;
        let zs2v = SCILSZs2::try_from(value[5])?;
        let depreciation_information = SCILSDepreciationInformation::try_from(value[6])?;
        let downstream_driveway_information =
            SCILSDrivewayInformation::try_from((value[7] & 0xF0) >> 4)?;
        let upstream_driveway_information = SCILSDrivewayInformation::try_from(value[7] & 0x0F)?;
        let dark_switching = SCILSDarkSwitching::try_from(value[8])?;
        let mut nationally_specified_information = [0; 9];
        nationally_specified_information[..].copy_from_slice(&value[9..18]);
        Ok(Self {
            main,
            additional,
            zs3,
            zs3v,
            zs2,
            zs2v,
            depreciation_information,
            upstream_driveway_information,
            downstream_driveway_information,
            dark_switching,
            nationally_specified_information,
        })
    }
}

impl SCITelegram {
    pub fn scils_show_signal_aspect(
        sender: &str,
        receiver: &str,
        signal_aspect: SCILSSignalAspect,
    ) -> Self {
        Self {
            protocol_type: ProtocolType::SCIProtocolLS,
            message_type: SCIMessageType::scils_show_signal_aspect(),
            sender: sender.to_string(),
            receiver: receiver.to_string(),
            payload: signal_aspect.into(),
        }
    }

    pub fn scils_change_brightness(
        sender: &str,
        receiver: &str,
        brightness: SCILSBrightness,
    ) -> Self {
        Self {
            protocol_type: ProtocolType::SCIProtocolLS,
            message_type: SCIMessageType::scils_change_brightness(),
            sender: sender.to_string(),
            receiver: receiver.to_string(),
            payload: SCIPayload::from_slice(&[brightness as u8]),
        }
    }

    pub fn scils_signal_aspect_status(
        sender: &str,
        receiver: &str,
        signal_aspect: SCILSSignalAspect,
    ) -> Self {
        Self {
            protocol_type: ProtocolType::SCIProtocolLS,
            message_type: SCIMessageType::scils_signal_aspect_status(),
            sender: sender.to_string(),
            receiver: receiver.to_string(),
            payload: signal_aspect.into(),
        }
    }

    pub fn scils_brightness_status(
        sender: &str,
        receiver: &str,
        brightness: SCILSBrightness,
    ) -> Self {
        Self {
            protocol_type: ProtocolType::SCIProtocolLS,
            message_type: SCIMessageType::scils_brightness_status(),
            sender: sender.to_string(),
            receiver: receiver.to_string(),
            payload: SCIPayload::from_slice(&[brightness as u8]),
        }
    }
}
