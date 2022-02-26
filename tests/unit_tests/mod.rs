pub mod sip_server;

pub fn debug(udp_tuple: &models::transport::UdpTuple) {
    println!(
        "\nRequest from {}: \n{}",
        udp_tuple.peer,
        String::from_utf8(udp_tuple.bytes.to_vec()).expect("string")
    );
}
