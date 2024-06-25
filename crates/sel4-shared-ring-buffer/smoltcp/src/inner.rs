//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use core::alloc::Layout;
use core::ops::Range;
use core::ptr::NonNull;

use smoltcp::phy::DeviceCapabilities;

use sel4_bounce_buffer_allocator::{AbstractBounceBufferAllocator, BounceBufferAllocator};
use sel4_externally_shared::ExternallySharedRef;
use sel4_shared_ring_buffer::{
    roles::Provide, Descriptor, PeerMisbehaviorError as SharedRingBuffersPeerMisbehaviorError,
    RingBuffers,
};
use sel4_shared_ring_buffer_bookkeeping::slot_tracker::*;

pub(crate) struct Inner<A> {
    dma_region: ExternallySharedRef<'static, [u8]>,
    bounce_buffer_allocator: BounceBufferAllocator<A>,
    rx_ring_buffers: RingBuffers<'static, Provide, fn()>,
    tx_ring_buffers: RingBuffers<'static, Provide, fn()>,
    rx_buffers: SlotTracker<RxStateTypesImpl>,
    tx_buffers: SlotTracker<TxStateTypesImpl>,
    caps: DeviceCapabilities,
}

pub(crate) type RxBufferIndex = usize;

enum RxStateTypesImpl {}

impl SlotStateTypes for RxStateTypesImpl {
    type Common = Descriptor;
    type Free = RxFree;
    type Occupied = RxOccupied;
}

struct RxFree {
    len: usize,
}

enum RxOccupied {
    Waiting,
    Claimed { len: usize },
}

pub(crate) type TxBufferIndex = usize;

enum TxStateTypesImpl {}

impl SlotStateTypes for TxStateTypesImpl {
    type Common = ();
    type Free = ();
    type Occupied = TxOccupied;
}

enum TxOccupied {
    Claimed,
    Sent { range: Range<usize> },
}

impl<A: AbstractBounceBufferAllocator> Inner<A> {
    pub(crate) fn new(
        dma_region: ExternallySharedRef<'static, [u8]>,
        mut bounce_buffer_allocator: BounceBufferAllocator<A>,
        mut rx_ring_buffers: RingBuffers<'static, Provide, fn()>,
        tx_ring_buffers: RingBuffers<'static, Provide, fn()>,
        num_rx_buffers: usize,
        rx_buffer_size: usize,
        caps: DeviceCapabilities,
    ) -> Result<Self, Error> {
        let rx_buffers = SlotTracker::new_occupied((0..num_rx_buffers).map(|i| {
            let range = bounce_buffer_allocator
                .allocate(Layout::from_size_align(rx_buffer_size, 1).unwrap())
                .map_err(|_| Error::BounceBufferAllocationError)
                .unwrap();
            let desc = Descriptor::from_encoded_addr_range(range, i);
            rx_ring_buffers
                .free_mut()
                .enqueue_and_commit(desc)
                .unwrap()
                .unwrap();
            (desc, RxOccupied::Waiting)
        }));

        let tx_buffers = SlotTracker::new_with_capacity((), (), tx_ring_buffers.free().capacity());

        Ok(Self {
            dma_region,
            bounce_buffer_allocator,
            rx_ring_buffers,
            tx_ring_buffers,
            rx_buffers,
            tx_buffers,
            caps,
        })
    }

    pub(crate) fn caps(&self) -> &DeviceCapabilities {
        &self.caps
    }

    pub(crate) fn poll(&mut self) -> Result<bool, PeerMisbehaviorError> {
        let mut notify_rx = false;

        while let Some(desc) = self.rx_ring_buffers.used_mut().dequeue()? {
            let ix = desc.cookie();
            if ix >= self.rx_buffers.capacity() {
                return Err(PeerMisbehaviorError::OutOfBoundsCookie);
            }

            let provided_desc = self.rx_buffers.get_common_value(ix).unwrap();
            if desc.encoded_addr() != provided_desc.encoded_addr()
                || desc.len() > provided_desc.len()
            {
                return Err(PeerMisbehaviorError::DescriptorMismatch);
            }

            if !matches!(
                self.rx_buffers.get_state_value(ix).unwrap(),
                SlotStateValueRef::Occupied(RxOccupied::Waiting)
            ) {
                return Err(PeerMisbehaviorError::StateMismatch);
            }

            self.rx_buffers
                .free(
                    ix,
                    RxFree {
                        len: desc.encoded_addr_range().len(),
                    },
                )
                .unwrap();

            notify_rx = true;
        }

        if notify_rx {
            self.rx_ring_buffers.notify();
        }

        let mut notify_tx = false;

        while let Some(desc) = self.tx_ring_buffers.used_mut().dequeue()? {
            let ix = desc.cookie();

            let state_value = self
                .tx_buffers
                .get_state_value(ix)
                .map_err(|_| PeerMisbehaviorError::OutOfBoundsCookie)?;

            match state_value {
                SlotStateValueRef::Occupied(TxOccupied::Sent { range }) => {
                    self.bounce_buffer_allocator.deallocate(range.clone());
                }
                _ => {
                    return Err(PeerMisbehaviorError::StateMismatch);
                }
            }

            self.tx_buffers.free(ix, ()).unwrap();

            notify_tx = true;
        }

        if notify_tx {
            self.tx_ring_buffers.notify();
        }

        Ok(notify_rx || notify_tx)
    }

    pub(crate) fn can_receive(&mut self) -> bool {
        self.can_claim_rx_buffer() && self.can_claim_tx_buffer()
    }

    pub(crate) fn can_transmit(&mut self) -> bool {
        self.can_claim_tx_buffer()
    }

    pub(crate) fn receive(&mut self) -> Option<(RxBufferIndex, TxBufferIndex)> {
        if self.can_receive() {
            let rx = self.claim_rx_buffer().unwrap();
            let tx = self.claim_tx_buffer().unwrap();
            Some((rx, tx))
        } else {
            None
        }
    }

    pub(crate) fn transmit(&mut self) -> Option<TxBufferIndex> {
        self.claim_tx_buffer()
    }

    fn can_claim_rx_buffer(&self) -> bool {
        self.rx_buffers.num_free() > 0
    }

    fn claim_rx_buffer(&mut self) -> Option<RxBufferIndex> {
        let len = self.rx_buffers.peek_next_free_value()?.len;
        let (ix, _) = self.rx_buffers.occupy(RxOccupied::Claimed { len }).unwrap();
        Some(ix)
    }

    fn can_claim_tx_buffer(&self) -> bool {
        self.tx_buffers.num_free() > 0
    }

    fn claim_tx_buffer(&mut self) -> Option<TxBufferIndex> {
        let (ix, _) = self.tx_buffers.occupy(TxOccupied::Claimed)?;
        Some(ix)
    }

    pub(crate) fn consume_rx_start(&mut self, index: RxBufferIndex) -> NonNull<[u8]> {
        let desc = self.rx_buffers.get_common_value(index).unwrap();
        let start = desc.encoded_addr_range().start;
        let len = match self
            .rx_buffers
            .get_state_value(index)
            .unwrap()
            .as_occupied()
            .unwrap()
        {
            RxOccupied::Claimed { len } => len,
            _ => panic!(),
        };
        self.dma_region
            .as_mut_ptr()
            .index(start..(start + len))
            .as_raw_ptr()
    }

    pub(crate) fn consume_rx_finish(&mut self, _index: RxBufferIndex) {
        // nothing to do, for now
    }

    pub(crate) fn drop_rx(&mut self, index: RxBufferIndex) -> Result<(), PeerMisbehaviorError> {
        let occupied = self
            .rx_buffers
            .get_state_value_mut(index)
            .unwrap()
            .as_occupied()
            .unwrap();
        assert!(matches!(occupied, RxOccupied::Claimed { .. }));
        *occupied = RxOccupied::Waiting;
        let desc = self.rx_buffers.get_common_value(index).unwrap();
        self.rx_ring_buffers
            .free_mut()
            .enqueue_and_commit(*desc)?
            .unwrap();
        self.rx_ring_buffers.notify();
        Ok(())
    }

    pub(crate) fn consume_tx<F, R>(
        &mut self,
        index: TxBufferIndex,
        len: usize,
        f: F,
    ) -> Result<R, Error>
    where
        F: FnOnce(&mut [u8]) -> R,
    {
        let range = self
            .bounce_buffer_allocator
            .allocate(Layout::from_size_align(len, 1).unwrap())
            .map_err(|_| Error::BounceBufferAllocationError)?;

        let occupied = self
            .tx_buffers
            .get_state_value_mut(index)
            .unwrap()
            .as_occupied()
            .unwrap();
        assert!(matches!(occupied, TxOccupied::Claimed { .. }));
        *occupied = TxOccupied::Sent {
            range: range.clone(),
        };

        let mut ptr = self
            .dma_region
            .as_mut_ptr()
            .index(range.clone())
            .as_raw_ptr();
        let r = f(unsafe { ptr.as_mut() });

        let desc = Descriptor::from_encoded_addr_range(range, index);
        self.tx_ring_buffers
            .free_mut()
            .enqueue_and_commit(desc)?
            .unwrap();
        self.tx_ring_buffers.notify();

        Ok(r)
    }

    pub(crate) fn drop_tx(&mut self, index: TxBufferIndex) {
        let occupied = self
            .tx_buffers
            .get_state_value(index)
            .unwrap()
            .as_occupied()
            .unwrap();
        match occupied {
            TxOccupied::Claimed => {
                self.tx_buffers.free(index, ()).unwrap();
            }
            TxOccupied::Sent { .. } => {}
        }
    }
}

// // //

#[derive(Debug, Clone)]
pub enum Error {
    BounceBufferAllocationError,
    PeerMisbehaviorError(PeerMisbehaviorError),
}

#[derive(Debug, Clone)]
pub enum PeerMisbehaviorError {
    DescriptorMismatch,
    OutOfBoundsCookie,
    StateMismatch,
    SharedRingBuffersPeerMisbehaviorError(SharedRingBuffersPeerMisbehaviorError),
}

impl From<PeerMisbehaviorError> for Error {
    fn from(err: PeerMisbehaviorError) -> Self {
        Self::PeerMisbehaviorError(err)
    }
}

impl From<SharedRingBuffersPeerMisbehaviorError> for PeerMisbehaviorError {
    fn from(err: SharedRingBuffersPeerMisbehaviorError) -> Self {
        Self::SharedRingBuffersPeerMisbehaviorError(err)
    }
}

impl From<SharedRingBuffersPeerMisbehaviorError> for Error {
    fn from(err: SharedRingBuffersPeerMisbehaviorError) -> Self {
        PeerMisbehaviorError::from(err).into()
    }
}
