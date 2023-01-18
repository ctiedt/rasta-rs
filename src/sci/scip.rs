use crate::{RastaConnection, RastaListener};

pub enum SCIPointTargetLocation {
    PointLocationChangeToRight = 0x01,
    PointLocationChangeToLeft = 0x02,
}

pub enum SCIPointLocation {
    PointLocationRight = 0x01,
    PointLocationLeft = 0x02,
    PointNoTargetLocation = 0x03,
    PointBumped = 0x04,
}

pub struct SCIPListener {
    listener: RastaListener,
}

pub struct SCIPConnection {
    conn: RastaConnection,
}
