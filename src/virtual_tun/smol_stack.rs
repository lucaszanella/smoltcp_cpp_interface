//use smoltcp_openvpn_bridge::virtual_tun::VirtualTunInterface;
use super::interface::{CBuffer, CIpv4Address, CIpv4Cidr, CIpv6Address, CIpv6Cidr};
use super::virtual_tun::VirtualTunInterface as TunDevice;
use smoltcp::iface::{Interface, InterfaceBuilder, Routes};
use smoltcp::phy::wait as phy_wait;
use smoltcp::phy::{self, Device};
use std::os::unix::io::AsRawFd;

use smoltcp::socket::{
    AnySocket, RawSocket, RawSocketBuffer, Socket, SocketHandle, SocketRef, SocketSet, TcpSocket,
    TcpSocketBuffer, UdpSocket, UdpSocketBuffer,
};
use smoltcp::storage::PacketMetadata;
use smoltcp::time::Instant;
use smoltcp::wire::{
    IpAddress, IpCidr, IpEndpoint, IpProtocol, IpVersion, Ipv4Address, Ipv6Address,
};
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::ffi::c_void;
use std::ptr;
use std::rc::Rc;
use std::slice;
use std::sync::{Arc, Condvar, Mutex};

#[derive(PartialEq, Clone)]
pub enum SocketType {
    RAW_IPV4,
    RAW_IPV6,
    ICMP,
    TCP,
    UDP,
}

pub struct Blob {
    pub data: Vec<u8>,
    pub start: usize,
    //A pointer do the object (SmolOwner in C++) that owns the data on the slice
    pub pointer_to_owner: Option<*const c_void>,
    /*
        Function pointer to the function that receives the pointer_to_owner
        and deletes it, thus callings its destructor which deletes the owner
        of the data on the slice, which deletes the data on the slice
    */
    pub pointer_to_destructor: Option<unsafe extern "C" fn(*const c_void) -> u8>,
}

pub struct Packet {
    pub blob: Blob,
    pub endpoint: Option<IpEndpoint>,
}

impl<'a> Drop for Blob {
    fn drop(&mut self) {
        let f = self.pointer_to_destructor;
        match self.pointer_to_destructor {
            Some(f) => {
                unsafe { f(self.pointer_to_owner.unwrap()) };
            }
            None => {}
        }
        //println!("blob drop result: {}", r);
    }
}

pub struct SmolSocket {
    pub socket_type: SocketType,
    //Socket number inside SmolStack
    pub socket_handle: SocketHandle,
    pub to_send: Arc<Mutex<VecDeque<Packet>>>,
    //If we couldn't send entire packet at once, hold it here for next send
    current_to_send: Option<Packet>,
    pub received: Arc<Mutex<VecDeque<Vec<u8>>>>,
}

impl<'a> SmolSocket {
    pub fn new(socket_handle: SocketHandle, socket_type: SocketType) -> SmolSocket {
        SmolSocket {
            socket_type: socket_type,
            socket_handle: socket_handle,
            to_send: Arc::new(Mutex::new(VecDeque::new())),
            current_to_send: None,
            received: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    pub fn send(&mut self, packet: Packet) -> u8 {
        if packet.endpoint.is_none()
            && (self.socket_type == SocketType::UDP || self.socket_type == SocketType::ICMP)
        {
            panic!("this socket type needs an endpoint to send to");
        }
        self.to_send.lock().unwrap().push_back(packet);
        0
    }
    //TODO: figure out a better way than copying. Inneficient receive
    pub fn receive(
        &mut self,
        cbuffer: *mut CBuffer,
        allocate_function: extern "C" fn(size: usize) -> *mut u8,
    ) -> u8 {
        let s;
        {
            //Create a scope so we hold the queue for the least ammount needed
            //TODO: do I really need to create a scope?
            s = self.received.lock().unwrap().pop_front()
        }
        match s {
            Some(s) => {
                let p: *mut u8 = allocate_function(s.len());
                unsafe { ptr::copy(s.as_ptr(), p, s.len()) };
                unsafe {
                    *cbuffer = CBuffer {
                        data: p,
                        len: s.len(),
                    };
                }
                0
            }
            None => 1,
        }
    }

    pub fn get_latest_packet(&mut self) -> Option<Packet> {
        //If the last step couldn't send the entire blob,
        //the packet is in `self.current_to_send`, so we return it again
        //otherwise we return a fresh packet from the queue
        match self.current_to_send.take() {
            Some(packet) => Some(packet),
            //TODO: verify assertion below
            //lock happens very birefly, so the list is not kept locked much time
            None => {
                let packet = self.to_send.lock().unwrap().pop_front();
                packet
            }
        }
    }
}

pub struct SmolStack<'a, 'b: 'a, 'c: 'a + 'b, DeviceT>
where
    DeviceT: for<'d> Device<'d>,
{
    /*
        'b and 'c are lifetimes for the internal buffers
        for the socket. 'a is the lifetime of the socket itself
    */
    pub sockets: SocketSet<'a, 'b, 'c>,
    current_key: usize,
    pub fd: Option<i32>,
    smol_sockets: HashMap<usize, SmolSocket>,
    pub device: Option<DeviceT>,
    ip_addrs: Option<std::vec::Vec<IpCidr>>,
    default_v4_gw: Option<Ipv4Address>,
    default_v6_gw: Option<Ipv6Address>,
    pub interface: Option<Interface<'a, 'b, 'c, DeviceT>>,
    //For TunInterface only. Couldn't think of a way to
    //create a specialized SmolStack for this case only
    packets_from_inside: Option<Arc<Mutex<VecDeque<Vec<u8>>>>>,
    packets_from_outside: Option<Arc<Mutex<VecDeque<Blob>>>>,
    has_data: Option<Arc<(Mutex<()>, Condvar)>>,
}

impl<'a, 'b: 'a, 'c: 'a + 'b, DeviceT> SmolStack<'a, 'b, 'c, DeviceT>
where
    DeviceT: for<'d> Device<'d>,
{
    pub fn new(
        device: DeviceT,
        fd: Option<i32>,
        packets_from_inside: Option<Arc<Mutex<VecDeque<Vec<u8>>>>>,
        packets_from_outside: Option<Arc<Mutex<VecDeque<Blob>>>>,
        has_data: Option<Arc<(Mutex<()>, Condvar)>>
    ) -> SmolStack<'a, 'b, 'c, DeviceT> {
        let socket_set = SocketSet::new(vec![]);
        let ip_addrs = std::vec::Vec::new();

        SmolStack {
            sockets: socket_set,
            current_key: 0,
            fd: fd,
            smol_sockets: HashMap::new(),
            device: Some(device),
            ip_addrs: Some(ip_addrs),
            default_v4_gw: None,
            default_v6_gw: None,
            interface: None,
            packets_from_inside: packets_from_inside,
            packets_from_outside: packets_from_outside,
            has_data: has_data,
        }
    }

    pub fn new_socket_handle_key(&mut self) -> usize {
        //TODO: panic when usize is about to overflow
        self.current_key += 1;
        self.current_key
    }

    pub fn add_socket(&mut self, socket_type: SocketType, smol_socket_handle: usize) -> u8 {
        match socket_type {
            SocketType::TCP => {
                let rx_buffer = TcpSocketBuffer::new(vec![0; 65000]);
                let tx_buffer = TcpSocketBuffer::new(vec![0; 65000]);
                let socket = TcpSocket::new(rx_buffer, tx_buffer);
                let handle = self.sockets.add(socket);
                let smol_socket = SmolSocket::new(handle, SocketType::TCP);
                self.smol_sockets.insert(smol_socket_handle, smol_socket);
                0
            }
            SocketType::UDP => {
                let rx_buffer = UdpSocketBuffer::new(Vec::new(), vec![0; 1024]);
                let tx_buffer = UdpSocketBuffer::new(Vec::new(), vec![0; 1024]);
                let socket = UdpSocket::new(rx_buffer, tx_buffer);
                let handle = self.sockets.add(socket);
                let smol_socket = SmolSocket::new(handle, SocketType::UDP);
                self.smol_sockets.insert(smol_socket_handle, smol_socket);
                0
            }
            /*
            SocketType::RAW_IPV4 => {
                let rx_buffer = RawSocketBuffer::new(Vec::new(), vec![0; 1024]);
                let tx_buffer = RawSocketBuffer::new(Vec::new(), vec![0; 1024]);
                //TODO: which protocol?
                let socket = RawSocket::new(IpVersion::Ipv4,IpProtocol::Tcp,rx_buffer, tx_buffer);
                self.sockets.add(socket);
            }

            SocketType::RAW_IPV6 => {
                let rx_buffer = RawSocketBuffer::new(Vec::new(), vec![0; 1024]);
                let tx_buffer = RawSocketBuffer::new(Vec::new(), vec![0; 1024]);
                //TODO: which protocol?
                let socket = RawSocket::new(IpVersion::Ipv4,IpProtocol::Tcp,rx_buffer, tx_buffer);
                self.sockets.add(socket);
            }
            */
            _ => {
                panic! {"wrong choice for socket type"}
            }
        }
    }

    pub fn get_smol_socket(&mut self, smol_socket_handle: usize) -> Option<&mut SmolSocket> {
        let smol_socket = self.smol_sockets.get_mut(&smol_socket_handle);
        smol_socket
    }

    pub fn tcp_connect_ipv4(
        &mut self,
        smol_socket_handle: usize,
        address: CIpv4Address,
        src_port: u16,
        dst_port: u16,
    ) -> u8 {
        println!(
            "gonna get smol socket with handle key {}",
            smol_socket_handle
        );
        let smol_socket_ = self.smol_sockets.get(&smol_socket_handle);
        match smol_socket_ {
            Some(smol_socket) => {
                let socket_handle = smol_socket.socket_handle;
                let mut socket = self.sockets.get::<TcpSocket>(socket_handle);
                let r = socket.connect((Into::<Ipv4Address>::into(address), dst_port), src_port);
                match r {
                    Ok(_) => {
                        println!("connection ok");
                        0
                    }
                    _ => {
                        println!("connection error");
                        2
                    }
                }
            }
            None => {
                println!("NO smol socket");
                1
            }
        }
    }

    pub fn tcp_connect_ipv6(
        &mut self,
        smol_socket_handle: usize,
        address: CIpv6Address,
        src_port: u16,
        dst_port: u16,
    ) -> u8 {
        let smol_socket_ = self.smol_sockets.get(&smol_socket_handle);
        match smol_socket_ {
            Some(smol_socket) => {
                let socket_handle = smol_socket.socket_handle;
                let mut socket = self.sockets.get::<TcpSocket>(socket_handle);
                let r = socket.connect((Into::<Ipv6Address>::into(address), dst_port), src_port);
                match r {
                    Ok(_) => 0,
                    _ => 2,
                }
            }
            None => 1,
        }
    }

    pub fn add_ipv4_address(&mut self, cidr: CIpv4Cidr) {
        self.ip_addrs.as_mut().unwrap().push(IpCidr::new(
            Into::<IpAddress>::into(cidr.address),
            cidr.prefix,
        ));
    }

    pub fn add_ipv6_address(&mut self, cidr: CIpv6Cidr) {
        self.ip_addrs.as_mut().unwrap().push(IpCidr::new(
            Into::<IpAddress>::into(cidr.address),
            cidr.prefix,
        ));
    }

    pub fn add_default_v4_gateway(&mut self, address: CIpv4Address) {
        self.default_v4_gw = Some(Into::<Ipv4Address>::into(address));
    }

    pub fn add_default_v6_gateway(&mut self, address: CIpv6Address) {
        self.default_v6_gw = Some(Into::<Ipv6Address>::into(address));
    }

    pub fn finalize(&mut self) -> u8 {
        let routes_storage = BTreeMap::new();
        let mut routes = Routes::new(routes_storage);
        //TODO: return C error if something is wrong, no unwrap
        routes
            .add_default_ipv4_route(self.default_v4_gw.unwrap())
            .unwrap();
        routes
            .add_default_ipv6_route(self.default_v6_gw.unwrap())
            .unwrap();
        let interface = InterfaceBuilder::new(self.device.take().unwrap())
            .ip_addrs(self.ip_addrs.take().unwrap())
            .routes(routes)
            .finalize();
        self.interface = Some(interface);
        0
    }

    pub fn poll(&mut self) -> u8 {
        let timestamp = Instant::now();
        match self
            .interface
            .as_mut()
            .unwrap()
            .poll(&mut self.sockets, timestamp)
        {
            Ok(_) => 0,
            Err(e) => {
                //debug!("poll error: {}",e);
                1
            }
        }
    }

    pub fn spin(&mut self, smol_socket_handle: usize) -> u8 {
        let smol_socket = self.smol_sockets.get_mut(&smol_socket_handle).unwrap();
        match smol_socket.socket_type {
            SocketType::TCP => {
                let mut socket = self.sockets.get::<TcpSocket>(smol_socket.socket_handle);
                let mut put_back = false;
                if socket.may_send() {
                    //Returns None if there are no packets
                    let mut packet = smol_socket.get_latest_packet();
                    match &mut packet {
                        Some(ref mut packet) => {
                            println!("some packet");
                            //Sends from the start (which might be more than 0 if we didn't send
                            //an entire packet in the last call)
                            let bytes_sent = socket
                                .send_slice(&packet.blob.data.as_slice()[packet.blob.start..]);
                            match bytes_sent {
                                Ok(bytes_sent) => {
                                    /*
                                        Sent less than entire packet, so we must put this packet
                                        in `smol_socket.current_to_send` so it's returned the next time
                                        so we can continue sending it
                                    */
                                    if bytes_sent < packet.blob.data.len() {
                                        let remaining_bytes = packet.blob.data.len() - bytes_sent;
                                        //start from remaining in the next call
                                        packet.blob.start = remaining_bytes;
                                        put_back = true;
                                    } else {
                                        //Sent the entire packet, nothing needs to be done
                                        //0
                                    }
                                }
                                Err(e) => {
                                    println!("bytes not sent, ERROR {}, putting packet back", e);
                                    //1
                                }
                            }
                        }
                        None => {}
                    }
                    //Outside of match because it matches as reference so we cannot move
                    if put_back {
                        smol_socket.current_to_send = packet;
                    }
                } else {
                    //1
                }
                if socket.can_recv() {
                    socket
                        .recv(|data| {
                            let len = data.len();
                            {
                                let mut s = vec![0; len];
                                s.copy_from_slice(data);
                                smol_socket.received.lock().unwrap().push_back(s);
                            }
                            (len, ())
                        })
                        .unwrap();
                //0
                } else {
                    //2
                }
                0
            }
            SocketType::UDP => panic!("not implemented yet"),
            //TODO
            SocketType::ICMP => panic!("not implemented yet"),
            SocketType::RAW_IPV4 => panic!("not implemented yet"),
            SocketType::RAW_IPV6 => panic!("not implemented yet"),
        }
    }

    //Send a packet to the stack (Ethernet/IP)
    //not to confuse with TCP/UDP/etc packets
    pub fn send(&mut self, blob: Blob) -> u8 {
        let packets_from_outside = &*self.packets_from_outside.as_ref().unwrap().clone();
        packets_from_outside.lock().unwrap().push_back(blob);
        0
    }

    /*
        TODO: figure out a better way than copying. Inneficient receive
    */
    //Receive a packet from the stack (Ethernet/IP)
    //not to confuse with TCP/UDP/etc packets
    //TODO: Rename to receive_wait()?
    pub fn receive_wait(
        &mut self,
        cbuffer: *mut CBuffer,
        allocate_function: extern "C" fn(size: usize) -> *mut u8,
    ) -> u8 {
        let s;
        //let has_data = &*self.has_data.as_ref().unwrap().clone();
        let packets_from_inside =
            &*self.packets_from_inside.as_ref().unwrap().clone();
        {
            //Create a scope so we hold the queue for the least ammount needed
            //TODO: do I really need to create a scope?

            //TODO: handle Mutex poisoning error
            //condition_variable.wait_while(packets_from_inside.lock().unwrap(), |p| p.len() > 0);
            s = packets_from_inside.lock().unwrap().pop_front();
        }
        match s {
            Some(s) => {
                //Allocates a raw pointer on C++ side
                let p: *mut u8 = allocate_function(s.len());
                //Fills the pointer
                unsafe { ptr::copy(s.as_ptr(), p, s.len()) };
                //Sends the pointer back to C++, which has the responsibility
                //to delete it
                unsafe {
                    *cbuffer = CBuffer {
                        data: p,
                        len: s.len(),
                    };
                }
                //0 means everything went well
                0
            }
            None => 1,
        }
    }

    /*
        Returns 0 in case of sucess
        Returns 1 if there's no packet to receive
    */
    pub fn receive_instantly(
        &mut self,
        cbuffer: *mut CBuffer,
        allocate_function: extern "C" fn(size: usize) -> *mut u8,
    ) -> u8 {
        let s;
        //We ignore the condvar because we want to return immediately
        let packets_from_inside = &*self.packets_from_inside.as_ref().unwrap().clone();
        {
            //Create a scope so we hold the queue for the least ammount needed
            //TODO: do I really need to create a scope?
            s = packets_from_inside.lock().unwrap().pop_front()
        }
        match s {
            Some(s) => {
                //Allocates a raw pointer on C++ side
                let p: *mut u8 = allocate_function(s.len());
                //Fills the pointer
                unsafe { ptr::copy(s.as_ptr(), p, s.len()) };
                //Sends the pointer back to C++, which has the responsibility
                //to delete it
                unsafe {
                    *cbuffer = CBuffer {
                        data: p,
                        len: s.len(),
                    };
                }
                0
            }
            None => 1,
        }
    }
}
