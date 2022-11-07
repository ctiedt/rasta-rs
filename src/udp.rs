extern {
    pub fn udp_bind(addr: *const u8) -> u32;
    pub fn udp_recv(buf: *mut u8, amount: usize, src: u32);
    pub fn udp_send(to: *const u8, data: *const u8);
}
