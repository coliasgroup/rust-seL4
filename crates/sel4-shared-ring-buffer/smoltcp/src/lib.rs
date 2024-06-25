//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]

extern crate alloc;

use alloc::rc::Rc;
use core::cell::RefCell;

use smoltcp::phy::{self, Device, DeviceCapabilities};
use smoltcp::time::Instant;

use sel4_bounce_buffer_allocator::{AbstractBounceBufferAllocator, BounceBufferAllocator};
use sel4_externally_shared::ExternallySharedRef;
use sel4_shared_ring_buffer::{roles::Provide, RingBuffers};

mod inner;

pub use inner::{Error, PeerMisbehaviorError};
use inner::{Inner, RxBufferIndex, TxBufferIndex};

pub struct DeviceImpl<A> {
    inner: Rc<RefCell<Inner<A>>>,
}

impl<A> Clone for DeviceImpl<A> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<A: AbstractBounceBufferAllocator> DeviceImpl<A> {
    pub fn new(
        dma_region: ExternallySharedRef<'static, [u8]>,
        bounce_buffer_allocator: BounceBufferAllocator<A>,
        rx_ring_buffers: RingBuffers<'static, Provide, fn()>,
        tx_ring_buffers: RingBuffers<'static, Provide, fn()>,
        num_rx_buffers: usize,
        rx_buffer_size: usize,
        caps: DeviceCapabilities,
    ) -> Result<Self, Error> {
        Ok(Self {
            inner: Rc::new(RefCell::new(Inner::new(
                dma_region,
                bounce_buffer_allocator,
                rx_ring_buffers,
                tx_ring_buffers,
                num_rx_buffers,
                rx_buffer_size,
                caps,
            )?)),
        })
    }

    fn inner(&self) -> &Rc<RefCell<Inner<A>>> {
        &self.inner
    }

    pub fn poll(&self) -> bool {
        self.inner().borrow_mut().poll().unwrap()
    }

    fn new_rx_token(&self, rx_buffer: RxBufferIndex) -> RxToken<A> {
        RxToken {
            buffer: rx_buffer,
            shared: self.clone(),
        }
    }

    fn new_tx_token(&self, tx_buffer: TxBufferIndex) -> TxToken<A> {
        TxToken {
            buffer: tx_buffer,
            shared: self.clone(),
        }
    }
}

impl<A: AbstractBounceBufferAllocator> Device for DeviceImpl<A> {
    type RxToken<'a> = RxToken<A> where A: 'a;
    type TxToken<'a> = TxToken<A> where A: 'a;

    fn capabilities(&self) -> DeviceCapabilities {
        self.inner().borrow().caps().clone()
    }

    fn receive(&mut self, _timestamp: Instant) -> Option<(Self::RxToken<'_>, Self::TxToken<'_>)> {
        self.inner()
            .borrow_mut()
            .receive()
            .map(|(rx_ix, tx_ix)| (self.new_rx_token(rx_ix), self.new_tx_token(tx_ix)))
    }

    fn transmit(&mut self, _timestamp: Instant) -> Option<Self::TxToken<'_>> {
        self.inner()
            .borrow_mut()
            .transmit()
            .map(|ix| self.new_tx_token(ix))
    }
}

pub struct RxToken<A: AbstractBounceBufferAllocator> {
    buffer: RxBufferIndex,
    shared: DeviceImpl<A>,
}

impl<A: AbstractBounceBufferAllocator> phy::RxToken for RxToken<A> {
    fn consume<R, F>(self, f: F) -> R
    where
        F: FnOnce(&mut [u8]) -> R,
    {
        let mut ptr = self
            .shared
            .inner()
            .borrow_mut()
            .consume_rx_start(self.buffer);
        let r = f(unsafe { ptr.as_mut() });
        self.shared
            .inner()
            .borrow_mut()
            .consume_rx_finish(self.buffer);
        r
    }
}

impl<A: AbstractBounceBufferAllocator> Drop for RxToken<A> {
    fn drop(&mut self) {
        self.shared
            .inner()
            .borrow_mut()
            .drop_rx(self.buffer)
            .unwrap()
    }
}

pub struct TxToken<A: AbstractBounceBufferAllocator> {
    buffer: TxBufferIndex,
    shared: DeviceImpl<A>,
}

impl<A: AbstractBounceBufferAllocator> phy::TxToken for TxToken<A> {
    fn consume<R, F>(self, len: usize, f: F) -> R
    where
        F: FnOnce(&mut [u8]) -> R,
    {
        self.shared
            .inner()
            .borrow_mut()
            .consume_tx(self.buffer, len, f)
            .unwrap()
    }
}

impl<A: AbstractBounceBufferAllocator> Drop for TxToken<A> {
    fn drop(&mut self) {
        self.shared.inner().borrow_mut().drop_tx(self.buffer)
    }
}
