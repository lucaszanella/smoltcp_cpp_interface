#![allow(unsafe_code)]
#![allow(unused)]

use super::smol_stack::Blob;
use smoltcp::phy::{self, Device, DeviceCapabilities, Medium};
use smoltcp::time::Instant;
use smoltcp::{Error, Result};
use std::cell::RefCell;
use std::collections::VecDeque;
use std::io;
use std::rc::Rc;
use std::sync::{Arc, Condvar, Mutex};
use std::vec::Vec;
use std::time::Duration;
use std::isize;
use std::ops::Deref;
use std::slice;

static ERR_WOULD_BLOCK: u32 = 1;
/// A virtual TUN interface.

#[derive(Clone)]
pub struct VirtualTunInterface {
    mtu: usize,
    has_data: Arc<(Mutex<()>, Condvar)>,
    packets_from_inside: Arc<Mutex<VecDeque<Vec<u8>>>>,
    packets_from_outside: Arc<Mutex<VecDeque<Blob>>>,
}

impl<'a> VirtualTunInterface {
    pub fn new(
        _name: &str,
        packets_from_inside: Arc<Mutex<VecDeque<Vec<u8>>>>,
        packets_from_outside: Arc<Mutex<VecDeque<Blob>>>,
        has_data: Arc<(Mutex<()>, Condvar)>
    ) -> Result<VirtualTunInterface> {
        let mtu = 1500; //??
        Ok(VirtualTunInterface {
            mtu: mtu,
            has_data: has_data,
            packets_from_outside: packets_from_outside,
            packets_from_inside: packets_from_inside,
        })
    }
    //TODO: this cant block, I guess?? Or it can..
    fn recv(&mut self, buffer: &mut [u8]) -> core::result::Result<usize, u32> {
        //TODO: should I clone?
        let packets_from_outside = &*self.packets_from_outside.clone();
        let p;
        {
            p = packets_from_outside.lock().unwrap().pop_front();
        }
        match p {
            Some(packet) => {
                buffer.copy_from_slice(packet.data.as_slice());
                let (mutex, has_data_condition_variable) = &*self.has_data.clone();
                has_data_condition_variable.notify_one();
                Ok(packet.data.len())
            }
            /*
                Simply returns ERR_WOULD_BLOCK. Device::receive(&mut self) is prepared
                to assume that it'd block so it does nothing in this case (returns None)
            */
            None => Err(ERR_WOULD_BLOCK),
        }
    }
}

impl<'d> Device<'d> for VirtualTunInterface {
    type RxToken = RxToken;
    type TxToken = TxToken;

    fn capabilities(&self) -> DeviceCapabilities {
        let mut d = DeviceCapabilities::default();
        d.max_transmission_unit = self.mtu;
        d
    }

    fn receive(&'d mut self) -> Option<(Self::RxToken, Self::TxToken)> {
        let mut buffer = vec![0; self.mtu];
        match self.recv(&mut buffer[..]) {
            Ok(size) => {
                println!("virtual_tun receive size {}", size);
                buffer.resize(size, 0);
                let rx = RxToken {
                    lower: Rc::new(RefCell::new(self.clone())),
                    buffer,
                };
                let tx = TxToken {
                    lower: Rc::new(RefCell::new(self.clone())),
                };
                Some((rx, tx))
            }
            //Simulates a tun/tap device that returns EWOULDBLOCK
            Err(err) if err == ERR_WOULD_BLOCK => None,
            Err(err) => panic!("{}", err),
        }
    }

    fn transmit(&'d mut self) -> Option<Self::TxToken> {
        println!("virtual_tun transmit");

        Some(TxToken {
            lower: Rc::new(RefCell::new(self.clone())),
        })
    }

    fn medium(&self) -> Medium {
        Medium::Ip
    }
}

#[doc(hidden)]
pub struct RxToken {
    lower: Rc<RefCell<VirtualTunInterface>>,
    buffer: Vec<u8>,
}

impl phy::RxToken for RxToken {
    fn consume<R, F>(mut self, _timestamp: Instant, f: F) -> Result<R>
    where
        F: FnOnce(&mut [u8]) -> Result<R>,
    {
        let mut lower = self.lower.as_ref().borrow_mut();
        let r = f(&mut self.buffer[..]);
        let (mutex, has_data_condition_variable) = &*lower.has_data.clone();
        has_data_condition_variable.notify_one();
        r
    }
}

#[doc(hidden)]
pub struct TxToken {
    lower: Rc<RefCell<VirtualTunInterface>>,
}

impl<'a> phy::TxToken for TxToken {
    fn consume<R, F>(self, _timestamp: Instant, len: usize, f: F) -> Result<R>
    where
        F: FnOnce(&mut [u8]) -> Result<R>,
    {
        let mut lower = self.lower.as_ref().borrow_mut();
        let mut buffer = vec![0; len];
        let result = f(&mut buffer);
        
        let packets_from_inside = &*lower.packets_from_inside.clone();
        {
            packets_from_inside.lock().unwrap().push_back(buffer);
        }
        
        let (mutex, has_data_condition_variable) = &*lower.has_data.clone();
        has_data_condition_variable.notify_one();
        result
    }
}
