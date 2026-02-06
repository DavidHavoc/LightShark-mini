use crate::config::Config;
use crate::state::{PacketMetadata, TrafficState};
use etherparse::{NetSlice, SlicedPacket, TransportSlice};
use pcap::Device;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::mpsc::Sender;

/// Filter configuration for packet capture
#[derive(Clone, Debug, Default)]
pub struct FilterConfig {
    pub port: Option<u16>,
    pub ip: Option<String>,
    pub protocol: Option<String>,
}

impl From<&Config> for FilterConfig {
    fn from(config: &Config) -> Self {
        Self {
            port: config.filter_port,
            ip: config.filter_ip.clone(),
            protocol: config.filter_protocol.clone(),
        }
    }
}

impl FilterConfig {
    /// Check if a packet matches the filter criteria
    pub fn matches(&self, meta: &PacketMetadata) -> bool {
        // Port filter
        if let Some(port) = self.port {
            if meta.src_port != port && meta.dst_port != port {
                return false;
            }
        }

        // IP filter
        if let Some(ref ip) = self.ip {
            if meta.src_ip != *ip && meta.dst_ip != *ip {
                return false;
            }
        }

        // Protocol filter
        if let Some(ref proto) = self.protocol {
            if !meta.protocol.eq_ignore_ascii_case(proto) {
                return false;
            }
        }

        true
    }
}

pub fn start_sniffer(
    interface_name: Option<String>,
    tx: Sender<PacketMetadata>,
    running: Arc<AtomicBool>,
    traffic_state: Arc<TrafficState>,
    filter: FilterConfig,
    quiet: bool,
) {
    let device = if let Some(name) = interface_name {
        Device::list()
            .unwrap()
            .into_iter()
            .find(|d| d.name == name)
            .expect("Device not found")
    } else {
        Device::lookup().expect("Device lookup failed").expect("No default device")
    };

    if !quiet {
        println!("Capturing on device: {}", device.name);
        if filter.port.is_some() || filter.ip.is_some() || filter.protocol.is_some() {
            println!("Filters: port={:?}, ip={:?}, protocol={:?}", 
                filter.port, filter.ip, filter.protocol);
        }
    }

    let mut cap = pcap::Capture::from_device(device)
        .unwrap()
        .promisc(true)
        .snaplen(65535)
        .timeout(1000)
        .open()
        .unwrap();

    while running.load(Ordering::Relaxed) {
        match cap.next_packet() {
            Ok(packet) => {
                if let Ok(sliced) = SlicedPacket::from_ethernet(&packet.data) {
                    let mut meta = PacketMetadata {
                        timestamp: chrono::Utc::now().timestamp_millis(),
                        src_ip: "?.?.?.?".to_string(),
                        dst_ip: "?.?.?.?".to_string(),
                        src_port: 0,
                        dst_port: 0,
                        protocol: "Unknown".to_string(),
                        length: packet.header.len as usize,
                    };

                    match sliced.net {
                        Some(NetSlice::Ipv4(slice)) => {
                            let header = slice.header();
                            meta.src_ip = header.source_addr().to_string();
                            meta.dst_ip = header.destination_addr().to_string();
                            meta.protocol = "IPv4".to_string();
                        }
                        Some(NetSlice::Ipv6(slice)) => {
                            let header = slice.header();
                            meta.src_ip = header.source_addr().to_string();
                            meta.dst_ip = header.destination_addr().to_string();
                            meta.protocol = "IPv6".to_string();
                        }
                        _ => {}
                    }

                    match sliced.transport {
                        Some(TransportSlice::Tcp(header)) => {
                            meta.src_port = header.source_port();
                            meta.dst_port = header.destination_port();
                            meta.protocol = "TCP".to_string();
                        }
                        Some(TransportSlice::Udp(header)) => {
                            meta.src_port = header.source_port();
                            meta.dst_port = header.destination_port();
                            meta.protocol = "UDP".to_string();
                        }
                        _ => {}
                    }

                    // Apply filters
                    if meta.protocol != "Unknown" && filter.matches(&meta) {
                        traffic_state.update(&meta);
                        if let Err(_) = tx.blocking_send(meta) {
                            break;
                        }
                    }
                }
            }
            Err(pcap::Error::TimeoutExpired) => continue,
            Err(e) => {
                eprintln!("Packet capture error: {}", e);
            }
        }
    }
}
