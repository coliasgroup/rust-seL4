//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]

extern crate alloc;

use smoltcp::phy::{self, Device, DeviceCapabilities};
use smoltcp::time::Instant;

use sel4_bounce_buffer_allocator::{AbstractBounceBufferAllocator, BounceBufferAllocator};
use sel4_externally_shared::ExternallySharedRef;
use sel4_shared_ring_buffer::{roles::Provide, RingBuffers};
use sel4_sync_abstractions::{Shared, SharedContainer, SharedRcRefCell};

mod inner;

pub use inner::{Error, PeerMisbehaviorError};
use inner::{Inner, RxBufferIndex, TxBufferIndex};

pub struct DeviceImpl<A, S: Shared = SharedRcRefCell> {
    inner: S::Container<Inner<A>>,
}

impl<A, S: Shared> Clone for DeviceImpl<A, S> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<A: AbstractBounceBufferAllocator, S: Shared> DeviceImpl<A, S> {
    #[allow(private_interfaces)]
    pub fn new_with(
        init: <<S as Shared>::Container<Inner<A>> as SharedContainer<Inner<A>>>::Init,
        dma_region: ExternallySharedRef<'static, [u8]>,
        bounce_buffer_allocator: BounceBufferAllocator<A>,
        rx_ring_buffers: RingBuffers<'static, Provide, fn()>,
        tx_ring_buffers: RingBuffers<'static, Provide, fn()>,
        num_rx_buffers: usize,
        rx_buffer_size: usize,
        caps: DeviceCapabilities,
    ) -> Result<Self, Error> {
        Ok(Self {
            inner: S::Container::new(
                init,
                Inner::new(
                    dma_region,
                    bounce_buffer_allocator,
                    rx_ring_buffers,
                    tx_ring_buffers,
                    num_rx_buffers,
                    rx_buffer_size,
                    caps,
                )?,
            ),
        })
    }

    pub fn new(
        dma_region: ExternallySharedRef<'static, [u8]>,
        bounce_buffer_allocator: BounceBufferAllocator<A>,
        rx_ring_buffers: RingBuffers<'static, Provide, fn()>,
        tx_ring_buffers: RingBuffers<'static, Provide, fn()>,
        num_rx_buffers: usize,
        rx_buffer_size: usize,
        caps: DeviceCapabilities,
    ) -> Result<Self, Error> {
        Self::new_with(
            S::Container::DEFAULT_INIT,
            dma_region,
            bounce_buffer_allocator,
            rx_ring_buffers,
            tx_ring_buffers,
            num_rx_buffers,
            rx_buffer_size,
            caps,
        )
    }

    fn inner(&self) -> &S::Container<Inner<A>> {
        &self.inner
    }

    pub fn poll(&self) -> bool {
        self.inner().lock().poll().unwrap()
    }

    fn new_rx_token(&self, rx_buffer: RxBufferIndex) -> RxToken<A, S> {
        RxToken {
            buffer: rx_buffer,
            shared: self.clone(),
        }
    }

    fn new_tx_token(&self, tx_buffer: TxBufferIndex) -> TxToken<A, S> {
        TxToken {
            buffer: tx_buffer,
            shared: self.clone(),
        }
    }
}

impl<A: AbstractBounceBufferAllocator, S: Shared> Device for DeviceImpl<A, S> {
    type RxToken<'a> = RxToken<A, S> where A: 'a;
    type TxToken<'a> = TxToken<A, S> where A: 'a;

    fn capabilities(&self) -> DeviceCapabilities {
        self.inner().lock().caps().clone()
    }

    fn receive(&mut self, _timestamp: Instant) -> Option<(Self::RxToken<'_>, Self::TxToken<'_>)> {
        self.inner()
            .lock()
            .receive()
            .map(|(rx_ix, tx_ix)| (self.new_rx_token(rx_ix), self.new_tx_token(tx_ix)))
    }

    fn transmit(&mut self, _timestamp: Instant) -> Option<Self::TxToken<'_>> {
        self.inner()
            .lock()
            .transmit()
            .map(|ix| self.new_tx_token(ix))
    }
}

pub struct RxToken<A: AbstractBounceBufferAllocator, S: Shared> {
    buffer: RxBufferIndex,
    shared: DeviceImpl<A, S>,
}

impl<A: AbstractBounceBufferAllocator, S: Shared> phy::RxToken for RxToken<A, S> {
    fn consume<R, F>(self, f: F) -> R
    where
        F: FnOnce(&mut [u8]) -> R,
    {
        let mut ptr = self.shared.inner().lock().consume_rx_start(self.buffer);
        let r = f(unsafe { ptr.as_mut() });
        self.shared.inner().lock().consume_rx_finish(self.buffer);
        r
    }
}

impl<A: AbstractBounceBufferAllocator, S: Shared> Drop for RxToken<A, S> {
    fn drop(&mut self) {
        self.shared.inner().lock().drop_rx(self.buffer).unwrap()
    }
}

pub struct TxToken<A: AbstractBounceBufferAllocator, S: Shared> {
    buffer: TxBufferIndex,
    shared: DeviceImpl<A, S>,
}

impl<A: AbstractBounceBufferAllocator, S: Shared> phy::TxToken for TxToken<A, S> {
    fn consume<R, F>(self, len: usize, f: F) -> R
    where
        F: FnOnce(&mut [u8]) -> R,
    {
        self.shared
            .inner()
            .lock()
            .consume_tx(self.buffer, len, f)
            .unwrap()
    }
}

impl<A: AbstractBounceBufferAllocator, S: Shared> Drop for TxToken<A, S> {
    fn drop(&mut self) {
        self.shared.inner().lock().drop_tx(self.buffer)
    }
}
