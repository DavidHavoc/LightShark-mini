use crate::state::{PacketMetadata, TrafficState};
use etherparse::{NetSlice, SlicedPacket, TransportSlice};
use pcap::Device;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::mpsc::Sender;

pub fn start_sniffer(
    interface_name: Option<String>,
    tx: Sender<PacketMetadata>,
    running: Arc<AtomicBool>,
    traffic_state: Arc<TrafficState>,
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

    println!("Capturing on device: {}", device.name);

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
                    
                    if meta.protocol != "Unknown" {
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
