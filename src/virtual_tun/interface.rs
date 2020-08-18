extern crate rand;

use super::smol_stack::SmolSocket;
use super::smol_stack::{Blob, Packet, SmolStack, SocketType};
use super::virtual_tun::VirtualTunInterface as VirtualTunDevice;
use smoltcp::phy::wait as phy_wait;
use smoltcp::phy::TapInterface as TapDevice;
use smoltcp::phy::TunInterface as TunDevice;
use smoltcp::phy::TunInterface;
use smoltcp::socket::{SocketHandle, TcpSocket};
use smoltcp::time::Instant;
use smoltcp::wire::{IpAddress, IpCidr, IpEndpoint, Ipv4Address, Ipv6Address};
use std::collections::VecDeque;
use std::ffi::{c_void, CStr};
use std::os::raw::{c_char, c_int};
use std::os::unix::io::AsRawFd;
use std::slice;
use std::str::{self};
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;

pub enum SmolSocketType {
    VirtualTun,
    Tun,
}

#[repr(C)]
pub struct CBuffer {
    pub data: *mut u8,
    pub len: usize,
}

/*
    Proxy that switches the function call to the right
    instance based on socket type. Had to do this
    to support different Device types because SmolStack
    is templated on Device, because Interface (which is
    from smoltcp) is templated on Device
*/
pub enum SmolStackType<'a, 'b: 'a, 'c: 'a + 'b> {
    VirtualTun(SmolStack<'a, 'b, 'c, VirtualTunDevice>),
    Tun(SmolStack<'a, 'b, 'c, TunDevice>),
    Tap(SmolStack<'a, 'b, 'c, TapDevice>),
}

//TODO: erase when confirmed its working
impl<'a, 'b: 'a, 'c: 'a + 'b> Drop for SmolStackType<'a, 'b, 'c> {
    fn drop(&mut self) {
        println!("dropped SmolStackType");
    }
}

impl<'a, 'b: 'a, 'c: 'a + 'b> SmolStackType<'a, 'b, 'c> {
    pub fn new_virtual_tun(interface_name: String) -> Box<SmolStackType<'a, 'b, 'c>> {
        let packets_from_inside = Arc::new(Mutex::new(VecDeque::new()));
        let packets_from_outside = Arc::new(Mutex::new(VecDeque::new()));
        let has_data = Arc::new((Mutex::new(()), Condvar::new()));
        let device = VirtualTunDevice::new(
            interface_name.as_str(),
            packets_from_inside.clone(),
            packets_from_outside.clone(),
            has_data.clone(),
        )
        .unwrap();
        let smol_stack = SmolStack::new(
            device,
            None,
            Some(packets_from_inside.clone()),
            Some(packets_from_outside.clone()),
            Some(has_data.clone()),
        );
        Box::new(SmolStackType::VirtualTun(smol_stack))
    }

    pub fn new_tun(interface_name: String) -> Box<SmolStackType<'a, 'b, 'c>> {
        let device = TunDevice::new(interface_name.as_str()).unwrap();
        let has_data = Arc::new((Mutex::new(()), Condvar::new()));
        let fd = Some(device.as_raw_fd());
        let smol_stack = SmolStack::new(
            device,
            None,
            None,
            None,
            Some(has_data.clone()),
        );
        Box::new(SmolStackType::Tun(smol_stack))
    }

    pub fn new_tap(interface_name: String) -> Box<SmolStackType<'a, 'b, 'c>> {
        let device = TapDevice::new(interface_name.as_str()).unwrap();
        let has_data = Arc::new((Mutex::new(()), Condvar::new()));
        let fd = Some(device.as_raw_fd());
        let smol_stack = SmolStack::new(
            device,
            None,
            None,
            None,
            Some(has_data.clone()),
        );
        Box::new(SmolStackType::Tap(smol_stack))
    }

    pub fn new_socket_handle_key(&mut self) -> usize {
        match self {
            &mut SmolStackType::VirtualTun(ref mut smol_stack) => {
                smol_stack.new_socket_handle_key()
            }
            &mut SmolStackType::Tun(ref mut smol_stack) => smol_stack.new_socket_handle_key(),
            &mut SmolStackType::Tap(ref mut smol_stack) => smol_stack.new_socket_handle_key(),
        }
    }

    pub fn add_socket(&mut self, socket_type: SocketType, socket_handle: usize) -> u8 {
        match self {
            &mut SmolStackType::VirtualTun(ref mut smol_stack) => {
                smol_stack.add_socket(socket_type, socket_handle)
            }
            &mut SmolStackType::Tun(ref mut smol_stack) => {
                smol_stack.add_socket(socket_type, socket_handle)
            }
            &mut SmolStackType::Tap(ref mut smol_stack) => {
                smol_stack.add_socket(socket_type, socket_handle)
            }
        }
    }

    pub fn tcp_connect_ipv4(
        &mut self,
        socket_handle_key: usize,
        address: CIpv4Address,
        src_port: u16,
        dst_port: u16,
    ) -> u8 {
        match self {
            &mut SmolStackType::VirtualTun(ref mut smol_stack) => {
                smol_stack.tcp_connect_ipv4(socket_handle_key, address, src_port, dst_port)
            }
            &mut SmolStackType::Tun(ref mut smol_stack) => {
                smol_stack.tcp_connect_ipv4(socket_handle_key, address, src_port, dst_port)
            }
            &mut SmolStackType::Tap(ref mut smol_stack) => {
                smol_stack.tcp_connect_ipv4(socket_handle_key, address, src_port, dst_port)
            }
        }
    }

    pub fn tcp_connect(
        &mut self,
        socket_handle_key: usize,
        address: CIpAddress,
        src_port: u16,
        dst_port: u16,
    ) -> u8 {
        match self {
            &mut SmolStackType::VirtualTun(ref mut smol_stack) => {
                smol_stack.tcp_connect(socket_handle_key, address, src_port, dst_port)
            }
            &mut SmolStackType::Tun(ref mut smol_stack) => {
                smol_stack.tcp_connect(socket_handle_key, address, src_port, dst_port)
            }
            &mut SmolStackType::Tap(ref mut smol_stack) => {
                smol_stack.tcp_connect(socket_handle_key, address, src_port, dst_port)
            }
        }
    }
    
    pub fn may_send(
        &mut self,
        socket_handle_key: usize
    ) -> u8 {
        match self {
            &mut SmolStackType::VirtualTun(ref mut smol_stack) => {
                smol_stack.may_send(socket_handle_key)
            }
            &mut SmolStackType::Tun(ref mut smol_stack) => {
                smol_stack.may_send(socket_handle_key)
            }
            &mut SmolStackType::Tap(ref mut smol_stack) => {
                smol_stack.may_send(socket_handle_key)
            }
        }
    }
    

    pub fn get_smol_socket(&mut self, socket_handle_key: usize) -> Option<&mut SmolSocket> {
        match self {
            &mut SmolStackType::VirtualTun(ref mut smol_stack) => {
                smol_stack.get_smol_socket(socket_handle_key)
            }
            &mut SmolStackType::Tun(ref mut smol_stack) => {
                smol_stack.get_smol_socket(socket_handle_key)
            }
            &mut SmolStackType::Tap(ref mut smol_stack) => {
                smol_stack.get_smol_socket(socket_handle_key)
            }
        }
    }

    pub fn tcp_connect_ipv6(
        &mut self,
        socket_handle_key: usize,
        address: CIpv6Address,
        src_port: u16,
        dst_port: u16,
    ) -> u8 {
        match self {
            &mut SmolStackType::VirtualTun(ref mut smol_stack) => {
                smol_stack.tcp_connect_ipv6(socket_handle_key, address, src_port, dst_port)
            }
            &mut SmolStackType::Tun(ref mut smol_stack) => {
                smol_stack.tcp_connect_ipv6(socket_handle_key, address, src_port, dst_port)
            }
            &mut SmolStackType::Tap(ref mut smol_stack) => {
                smol_stack.tcp_connect_ipv6(socket_handle_key, address, src_port, dst_port)
            }
        }
    }

    pub fn add_ipv4_address(&mut self, cidr: CIpv4Cidr) {
        match self {
            &mut SmolStackType::VirtualTun(ref mut smol_stack) => smol_stack.add_ipv4_address(cidr),
            &mut SmolStackType::Tun(ref mut smol_stack) => smol_stack.add_ipv4_address(cidr),
            &mut SmolStackType::Tap(ref mut smol_stack) => smol_stack.add_ipv4_address(cidr),
        }
    }

    pub fn add_ipv6_address(&mut self, cidr: CIpv6Cidr) {
        match self {
            &mut SmolStackType::VirtualTun(ref mut smol_stack) => smol_stack.add_ipv6_address(cidr),
            &mut SmolStackType::Tun(ref mut smol_stack) => smol_stack.add_ipv6_address(cidr),
            &mut SmolStackType::Tap(ref mut smol_stack) => smol_stack.add_ipv6_address(cidr),
        }
    }

    pub fn add_default_v4_gateway(&mut self, address: CIpv4Address) {
        match self {
            &mut SmolStackType::VirtualTun(ref mut smol_stack) => {
                smol_stack.add_default_v4_gateway(address)
            }
            &mut SmolStackType::Tun(ref mut smol_stack) => {
                smol_stack.add_default_v4_gateway(address)
            }
            &mut SmolStackType::Tap(ref mut smol_stack) => {
                smol_stack.add_default_v4_gateway(address)
            }
        }
    }

    pub fn add_default_v6_gateway(&mut self, address: CIpv6Address) {
        match self {
            &mut SmolStackType::VirtualTun(ref mut smol_stack) => {
                smol_stack.add_default_v6_gateway(address)
            }
            &mut SmolStackType::Tun(ref mut smol_stack) => {
                smol_stack.add_default_v6_gateway(address)
            }
            &mut SmolStackType::Tap(ref mut smol_stack) => {
                smol_stack.add_default_v6_gateway(address)
            }
        }
    }

    pub fn finalize(&mut self) -> u8 {
        match self {
            &mut SmolStackType::VirtualTun(ref mut smol_stack) => smol_stack.finalize(),
            &mut SmolStackType::Tun(ref mut smol_stack) => smol_stack.finalize(),
            &mut SmolStackType::Tap(ref mut smol_stack) => smol_stack.finalize(),
        }
    }

    pub fn poll(&mut self) -> u8 {
        match self {
            &mut SmolStackType::VirtualTun(ref mut smol_stack) => smol_stack.poll(),
            &mut SmolStackType::Tun(ref mut smol_stack) => smol_stack.poll(),
            &mut SmolStackType::Tap(ref mut smol_stack) => smol_stack.poll(),
        }
    }

    pub fn spin(&mut self, socket_handle: usize) -> u8 {
        match self {
            &mut SmolStackType::VirtualTun(ref mut smol_stack) => smol_stack.spin(socket_handle),
            &mut SmolStackType::Tun(ref mut smol_stack) => smol_stack.spin(socket_handle),
            &mut SmolStackType::Tap(ref mut smol_stack) => smol_stack.spin(socket_handle),
        }
    }

    pub fn spin_all(&mut self) -> u8 {
        match self {
            &mut SmolStackType::VirtualTun(ref mut smol_stack) => smol_stack.spin_all(),
            &mut SmolStackType::Tun(ref mut smol_stack) => smol_stack.spin_all(),
            &mut SmolStackType::Tap(ref mut smol_stack) => smol_stack.spin_all(),
        }
    }

    pub fn phy_wait(&mut self, timestamp: i64) {
        match self {
            &mut SmolStackType::VirtualTun(ref mut smol_stack) => {
                smol_stack.phy_wait_timeout(Duration::from_millis(timestamp as u64))
            }
            &mut SmolStackType::Tun(ref mut smol_stack) => phy_wait(
                smol_stack.fd.unwrap(),
                smol_stack
                    .interface
                    .as_mut()
                    .unwrap()
                    .poll_delay(&smol_stack.sockets, Instant::from_millis(timestamp)),
            )
            .expect("wait error"),
            &mut SmolStackType::Tap(ref mut smol_stack) => phy_wait(
                smol_stack.fd.unwrap(),
                smol_stack
                    .interface
                    .as_mut()
                    .unwrap()
                    .poll_delay(&smol_stack.sockets, Instant::from_millis(timestamp)),
            )
            .expect("wait error"),
        }
    }

    pub fn receive_wait(
        &mut self,
        cbuffer: *mut CBuffer,
        allocate_function: extern "C" fn(size: usize) -> *mut u8,
    ) -> u8 {
        match self {
            &mut SmolStackType::VirtualTun(ref mut smol_stack) => smol_stack.receive_wait(cbuffer, allocate_function),
            _ => panic!("receive is only for VirtualTun")
            //&mut SmolStackType::Tun(ref mut smol_stack) => smol_stack.receive(cbuffer, allocate_function),
            //&mut SmolStackType::Tap(ref mut smol_stack) => smol_stack.receive(cbuffer, allocate_function),
        }
    }

    pub fn receive_instantly(
        &mut self,
        cbuffer: *mut CBuffer,
        allocate_function: extern "C" fn(size: usize) -> *mut u8,
    ) -> u8 {
        match self {
            &mut SmolStackType::VirtualTun(ref mut smol_stack) => smol_stack.receive_instantly(cbuffer, allocate_function),
            _ => panic!("receive is only for VirtualTun")
            //&mut SmolStackType::Tun(ref mut smol_stack) => smol_stack.receive(cbuffer, allocate_function),
            //&mut SmolStackType::Tap(ref mut smol_stack) => smol_stack.receive(cbuffer, allocate_function),
        }
    }

    pub fn send(&mut self, blob: Blob) -> u8 {
        match self {
            &mut SmolStackType::VirtualTun(ref mut smol_stack) => smol_stack.send(blob),
            _ => panic!("send is only for VirtualTun")
            //&mut SmolStackType::Tun(ref mut smol_stack) => smol_stack.receive(cbuffer, allocate_function),
            //&mut SmolStackType::Tap(ref mut smol_stack) => smol_stack.receive(cbuffer, allocate_function),
        }
    }
}

#[repr(C)]
pub struct CIpv4Address {
    pub address: [u8; 4],
}

#[repr(C)]
pub struct CIpAddress {
    pub is_ipv4: u8,
    pub ipv4_address: CIpv4Address,
    pub ipv6_address: CIpv6Address,

}

impl Into<Ipv4Address> for CIpv4Address {
    fn into(self) -> Ipv4Address {
        Ipv4Address::new(
            self.address[0],
            self.address[1],
            self.address[2],
            self.address[3],
        )
    }
}

impl Into<IpAddress> for CIpv4Address {
    fn into(self) -> IpAddress {
        IpAddress::v4(
            self.address[0],
            self.address[1],
            self.address[2],
            self.address[3],
        )
    }
}


impl Into<IpAddress> for CIpAddress {
    fn into(self) -> IpAddress {
        if self.is_ipv4==1 {
            self.ipv4_address.into()
        } else {
            self.ipv6_address.into()
        }
    }
}

#[repr(C)]
pub struct CIpv6Address {
    pub address: [u16; 8],
}

impl Into<Ipv6Address> for CIpv6Address {
    fn into(self) -> Ipv6Address {
        Ipv6Address::new(
            self.address[0],
            self.address[1],
            self.address[2],
            self.address[3],
            self.address[4],
            self.address[5],
            self.address[6],
            self.address[7],
        )
    }
}

impl Into<IpAddress> for CIpv6Address {
    fn into(self) -> IpAddress {
        IpAddress::v6(
            self.address[0],
            self.address[1],
            self.address[2],
            self.address[3],
            self.address[4],
            self.address[5],
            self.address[6],
            self.address[7],
        )
    }
}

//Warning: keep this synced with CIpEndpointType on interface.h
static CIpEndpoint_NONE: u8 = 0;
static CIPENDPOINT_IPV4: u8 = 1;
static CIPENDPOINT_IPV6: u8 = 0;

#[repr(C)]
pub struct CIpEndpoint {
    pub endpoint_type: u8,
    pub ipv4: CIpv4Address,
    pub ipv6: CIpv6Address,
    pub port: u16,
}

impl Into<Option<IpEndpoint>> for CIpEndpoint {
    fn into(self) -> Option<IpEndpoint> {
        if self.endpoint_type == CIPENDPOINT_IPV4 {
            Some(IpEndpoint::new(
                IpAddress::v4(
                    self.ipv4.address[0],
                    self.ipv4.address[1],
                    self.ipv4.address[2],
                    self.ipv4.address[3],
                ),
                self.port,
            ))
        } else if self.endpoint_type == CIPENDPOINT_IPV6 {
            Some(IpEndpoint::new(
                IpAddress::v6(
                    self.ipv6.address[0],
                    self.ipv6.address[1],
                    self.ipv6.address[2],
                    self.ipv6.address[3],
                    self.ipv6.address[4],
                    self.ipv6.address[5],
                    self.ipv6.address[6],
                    self.ipv6.address[7],
                ),
                self.port,
            ))
        } else {
            None
        }
    }
}

#[repr(C)]
pub struct CIpv4Cidr {
    pub address: CIpv4Address,
    pub prefix: u8,
}

#[repr(C)]
pub struct CIpv6Cidr {
    pub address: CIpv6Address,
    pub prefix: u8,
}

#[no_mangle]
pub extern "C" fn smol_stack_smol_stack_new_virtual_tun<'a, 'b: 'a, 'c: 'a + 'b>(
    interface_name: *const c_char,
) -> Box<SmolStackType<'a, 'b, 'c>> {
    let interface_name_c_str: &CStr = unsafe { CStr::from_ptr(interface_name) };
    let interface_name_slice: &str = interface_name_c_str.to_str().unwrap();
    let s: String = interface_name_slice.to_owned();
    SmolStackType::new_virtual_tun(s)
}

#[no_mangle]
pub extern "C" fn smol_stack_smol_stack_new_tun<'a, 'b: 'a, 'c: 'a + 'b>(
    interface_name: *const c_char,
) -> Box<SmolStackType<'a, 'b, 'c>> {
    let interface_name_c_str: &CStr = unsafe { CStr::from_ptr(interface_name) };
    let interface_name_slice: &str = interface_name_c_str.to_str().unwrap();
    let s: String = interface_name_slice.to_owned();
    SmolStackType::new_tun(s)
}

#[no_mangle]
pub extern "C" fn smol_stack_smol_stack_new_tap<'a, 'b: 'a, 'c: 'a + 'b>(
    interface_name: *const c_char,
) -> Box<SmolStackType<'a, 'b, 'c>> {
    let interface_name_c_str: &CStr = unsafe { CStr::from_ptr(interface_name) };
    let interface_name_slice: &str = interface_name_c_str.to_str().unwrap();
    let s: String = interface_name_slice.to_owned();
    SmolStackType::new_tap(s)
}

#[no_mangle]
pub extern "C" fn smol_stack_smol_socket_send(
    smol_stack: &mut SmolStackType,
    socket_handle_key: usize,
    data: *mut u8,
    len: usize,
    endpoint: CIpEndpoint,
    pointer_to_owner: *const c_void,
    pointer_to_destructor: unsafe extern "C" fn(*const c_void) -> u8,
) -> u8 {
    let smol_socket = smol_stack.get_smol_socket(socket_handle_key);
    //let packet_as_vector = unsafe { Vec::from_raw_parts(data, len, len) };
    let mut packet_as_vector = Vec::new();
    let slice = unsafe { slice::from_raw_parts(data, len) };
    packet_as_vector.extend_from_slice(slice);
    let packet = Packet {
        blob: Blob {
            data: packet_as_vector,
            start: 0,
            pointer_to_owner: Some(pointer_to_owner),
            pointer_to_destructor: Some(pointer_to_destructor),
        },
        endpoint: Into::<Option<IpEndpoint>>::into(endpoint),
    };
    match smol_socket {
        Some(smol_socket) => {
            smol_socket.send(packet);
            0
        }
        None => 1,
    }
}

/*
    Copies data instead of owning object that destructs things
*/
#[no_mangle]
pub extern "C" fn smol_stack_smol_socket_send_copy(
    smol_stack: &mut SmolStackType,
    socket_handle_key: usize,
    data: *mut u8,
    len: usize,
    endpoint: CIpEndpoint,
) -> u8 {
    let smol_socket = smol_stack.get_smol_socket(socket_handle_key);
    let mut packet_as_vector = Vec::new();
    let slice = unsafe { slice::from_raw_parts(data, len) };
    packet_as_vector.extend_from_slice(slice);
    let packet = Packet {
        blob: Blob {
            data: packet_as_vector,
            start: 0,
            pointer_to_owner: None,
            pointer_to_destructor: None,
        },
        endpoint: Into::<Option<IpEndpoint>>::into(endpoint),
    };
    match smol_socket {
        Some(smol_socket) => {
            smol_socket.send(packet);
            0
        }
        None => 1,
    }
}

#[no_mangle]
pub extern "C" fn smol_stack_smol_socket_receive(
    smol_stack: &mut SmolStackType,
    socket_handle_key: usize,
    cbuffer: *mut CBuffer,
    allocate_function: extern "C" fn(size: usize) -> *mut u8,
    address: *mut CIpAddress
) -> u8 {
    let smol_socket = smol_stack.get_smol_socket(socket_handle_key);
    match smol_socket {
        Some(smol_socket) => smol_socket.receive(cbuffer, allocate_function, address),
        None => 1,
    }
}

#[no_mangle]
pub extern "C" fn smol_stack_smol_socket_receive_wait(
    smol_stack: &mut SmolStackType,
    socket_handle_key: usize,
    cbuffer: *mut CBuffer,
    allocate_function: extern "C" fn(size: usize) -> *mut u8,
    address: *mut CIpAddress
) -> u8 {
    let smol_socket = smol_stack.get_smol_socket(socket_handle_key);
    match smol_socket {
        Some(smol_socket) => smol_socket.receive_wait(cbuffer, allocate_function, address),
        None => 1,
    }
}

#[no_mangle]
pub extern "C" fn smol_stack_smol_socket_may_send(
    smol_stack: &mut SmolStackType,
    socket_handle_key: usize,
) -> u8 {
    smol_stack.may_send(socket_handle_key)
}

#[no_mangle]
pub extern "C" fn smol_stack_add_socket(
    smol_stack: &mut SmolStackType,
    socket_type: u8,
    socket_handle: usize,
) -> u8 {
    match socket_type {
        0 => smol_stack.add_socket(SocketType::TCP, socket_handle),
        1 => smol_stack.add_socket(SocketType::UDP, socket_handle),
        _ => panic!("wrong type"),
    }
}

#[no_mangle]
pub extern "C" fn smol_stack_phy_wait(smol_stack: &mut SmolStackType, timestamp: i64) {
    smol_stack.phy_wait(timestamp)
}

#[no_mangle]
pub extern "C" fn smol_stack_tcp_connect(
    smol_stack: &mut SmolStackType,
    socket_handle_key: usize,
    address: CIpAddress,
    src_port: u16,
    dst_port: u16,
) -> u8 {
    smol_stack.tcp_connect(socket_handle_key, address, src_port, dst_port)
}

#[no_mangle]
pub extern "C" fn smol_stack_tcp_connect_ipv4(
    smol_stack: &mut SmolStackType,
    socket_handle_key: usize,
    address: CIpv4Address,
    src_port: u16,
    dst_port: u16,
) -> u8 {
    smol_stack.tcp_connect_ipv4(socket_handle_key, address, src_port, dst_port)
}

#[no_mangle]
pub extern "C" fn smol_stack_tcp_connect_ipv6(
    smol_stack: &mut SmolStackType,
    socket_handle_key: usize,
    address: CIpv6Address,
    src_port: u16,
    dst_port: u16,
) -> u8 {
    smol_stack.tcp_connect_ipv6(socket_handle_key, address, src_port, dst_port)
}

#[no_mangle]
pub extern "C" fn smol_stack_poll(smol_stack: &mut SmolStackType) -> u8 {
    smol_stack.poll()
}

#[no_mangle]
pub extern "C" fn smol_stack_spin(smol_stack: &mut SmolStackType, socket_handle: usize) -> u8 {
    smol_stack.spin(socket_handle)
}

#[no_mangle]
pub extern "C" fn smol_stack_spin_all(smol_stack: &mut SmolStackType) -> u8 {
    smol_stack.spin_all()
}

#[no_mangle]
pub extern "C" fn smol_stack_add_ipv4_address(smol_stack: &mut SmolStackType, cidr: CIpv4Cidr) {
    smol_stack.add_ipv4_address(cidr);
}

#[no_mangle]
pub extern "C" fn smol_stack_add_ipv6_address(smol_stack: &mut SmolStackType, cidr: CIpv6Cidr) {
    smol_stack.add_ipv6_address(cidr);
}

#[no_mangle]
pub extern "C" fn smol_stack_add_default_v4_gateway(
    smol_stack: &mut SmolStackType,
    address: CIpv4Address,
) {
    smol_stack.add_default_v4_gateway(address);
}

#[no_mangle]
pub extern "C" fn smol_stack_add_default_v6_gateway(
    smol_stack: &mut SmolStackType,
    address: CIpv6Address,
) {
    smol_stack.add_default_v6_gateway(address);
}

#[no_mangle]
pub extern "C" fn smol_stack_finalize<'a, 'b: 'a, 'c: 'a + 'b>(
    smol_stack: &mut SmolStackType<'a, 'b, 'c>,
) -> u8 {
    smol_stack.finalize()
}

#[no_mangle]
pub extern "C" fn smol_stack_destroy(_: Option<Box<SmolStackType>>) {}

#[no_mangle]
pub extern "C" fn smol_stack_virtual_tun_receive_instantly(
    smol_stack: &mut SmolStackType,
    cbuffer: *mut CBuffer,
    allocate_function: extern "C" fn(size: usize) -> *mut u8,
) -> u8 {
    smol_stack.receive_instantly(cbuffer, allocate_function)
}

#[no_mangle]
pub extern "C" fn smol_stack_virtual_tun_send(
    smol_stack: &mut SmolStackType,
    data: *mut u8,
    len: usize,
) -> u8 {
    let slice = unsafe { slice::from_raw_parts(data, len) };
    let mut packet_as_vector = Vec::new();
    packet_as_vector.extend_from_slice(slice);
    let blob = Blob {
        data: packet_as_vector,
        start: 0,
        pointer_to_owner: None,
        pointer_to_destructor: None,
    };
    smol_stack.send(blob)
}
