//! In-process packet broker boundary.

pub trait Broker<P> {
    fn publish(&mut self, packet: P);
    fn subscribe(&mut self, engine: &str, packet_type: &str);
    fn poll(&mut self, engine: &str) -> Option<P>;
    fn ack(&mut self, packet_id: &str);
    fn defer(&mut self, packet_id: &str, reason: &str);
    fn dead_letter(&mut self, packet_id: &str, reason: &str);
}
