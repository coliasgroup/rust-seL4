//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]

use core::array;
use core::borrow::BorrowMut;
use core::ops::Range;
use core::result;
use core::slice;

#[allow(unused_imports)]
use log::{debug, info, trace};

use sel4::{
    cap_type,
    init_thread::{self, Slot},
    CapRights, CapTypeForFrameObjectOfFixedSize,
};
use sel4_capdl_initializer_types::*;
use sel4_supervising::UserContextExt;

mod buffers;
mod cslot_allocator;
mod error;
mod hold_slots;
mod memory;

pub use buffers::{InitializerBuffers, PerObjectBuffer};
use cslot_allocator::{CSlotAllocator, CSlotAllocatorError};
pub use error::CapDLInitializerError;
use hold_slots::HoldSlots;
use memory::{get_user_image_frame_slot, CopyAddrs};

#[sel4::sel4_cfg(all(ARCH_RISCV64, not(PT_LEVELS = "3")))]
compile_error!("unsupported configuration");

type Result<T> = result::Result<T, CapDLInitializerError>;

pub struct Initializer<'a, N: ObjectName, D: Content, M: GetEmbeddedFrame, B> {
    bootinfo: &'a sel4::BootInfoPtr,
    user_image_bounds: Range<usize>,
    copy_addrs: CopyAddrs,
    spec_with_sources: &'a SpecWithSources<'a, N, D, M>,
    cslot_allocator: &'a mut CSlotAllocator,
    buffers: &'a mut InitializerBuffers<B>,
}

impl<'a, N: ObjectName, D: Content, M: GetEmbeddedFrame, B: BorrowMut<[PerObjectBuffer]>>
    Initializer<'a, N, D, M, B>
{
    pub fn initialize(
        bootinfo: &sel4::BootInfoPtr,
        user_image_bounds: Range<usize>,
        spec_with_sources: &SpecWithSources<N, D, M>,
        buffers: &mut InitializerBuffers<B>,
    ) -> ! {
        info!("Starting CapDL initializer");

        let copy_addrs = CopyAddrs::init(bootinfo, &user_image_bounds).unwrap();

        let mut cslot_allocator = CSlotAllocator::new(bootinfo.empty().range());

        Initializer {
            bootinfo,
            user_image_bounds,
            copy_addrs,
            spec_with_sources,
            cslot_allocator: &mut cslot_allocator,
            buffers,
        }
        .run()
        .unwrap_or_else(|err| panic!("Error: {}", err));

        info!("CapDL initializer done, suspending");

        init_thread::suspend_self()
    }

    fn spec(&self) -> &'a Spec<'a, N, D, M> {
        &self.spec_with_sources.spec
    }

    fn object_name(&self, indirect: &'a N) -> Option<&'a str> {
        indirect.object_name(self.spec_with_sources.object_name_source)
    }

    // // //

    fn run(&mut self) -> Result<()> {
        self.create_objects()?;

        self.init_irqs()?;
        self.init_asids()?;
        self.init_frames()?;
        self.init_vspaces()?;

        sel4::sel4_cfg_if! {
            if #[sel4_cfg(KERNEL_MCS)] {
                self.init_sched_contexts()?;
            }
        }

        self.init_tcbs()?;
        self.init_cspaces()?;

        self.start_threads()?;

        Ok(())
    }

    fn create_objects(&mut self) -> Result<()> {
        // This algorithm differs from that found in the upstream C CapDL
        // loader. In particular, this one is implemented with objects
        // specifying non-device paddrs in mind.

        debug!("Creating objects");

        // Sort untypeds by paddr
        let mut _uts_by_paddr_backing: [usize;
            sel4::sel4_cfg_usize!(MAX_NUM_BOOTINFO_UNTYPED_CAPS)] = array::from_fn(|i| i); // TODO (not a big deal) allocate in image rather than on stack
        let uts = self.bootinfo.untyped_list();
        let uts_by_paddr = {
            let arr = &mut _uts_by_paddr_backing[..uts.len()];
            arr.sort_unstable_by_key(|i| uts[*i].paddr());
            arr
        };

        // Index root objects

        let first_obj_without_paddr = self
            .spec()
            .root_objects()
            .partition_point(|named_obj| named_obj.object.paddr().is_some());
        let num_objs_with_paddr = first_obj_without_paddr;

        let mut by_size_start: [usize; sel4::WORD_SIZE] = array::from_fn(|_| 0);
        let mut by_size_end: [usize; sel4::WORD_SIZE] = array::from_fn(|_| 0);
        {
            for obj_id in first_obj_without_paddr..self.spec().root_objects().len() {
                let obj = &self.spec().object(obj_id);
                if let Some(blueprint) = obj.blueprint() {
                    by_size_end[blueprint.physical_size_bits()] += 1;
                }
            }
            let mut acc = first_obj_without_paddr;
            for (bits, n) in by_size_end.iter_mut().enumerate().rev() {
                by_size_start[bits] = acc;
                acc += *n;
                *n = acc;
            }
        }

        // In order to allocate objects which specify paddrs, we may have to
        // allocate dummies to manipulate watermarks. We must always retain at
        // least one reference to an object allocated from an untyped, or else
        // its watermark will reset. This juggling approach is an easy way to
        // ensure that we are always holding such a reference.
        let mut hold_slots = HoldSlots::new(self.cslot_allocator, cslot_to_relative_cptr)?;

        // Create root objects

        let mut next_obj_with_paddr = 0;
        for i_ut in uts_by_paddr.iter() {
            let ut = &uts[*i_ut];
            let ut_size_bits = ut.size_bits();
            let ut_size_bytes = 1 << ut_size_bits;
            let ut_paddr_start = ut.paddr();
            let ut_paddr_end = ut_paddr_start + ut_size_bytes;
            let mut cur_paddr = ut_paddr_start;
            trace!(
                "Allocating from untyped: {:#x}..{:#x} (size_bits = {}, device = {:?})",
                ut_paddr_start,
                ut_paddr_end,
                ut_size_bits,
                ut.is_device()
            );
            loop {
                let target = if next_obj_with_paddr < num_objs_with_paddr {
                    ut_paddr_end.min(self.spec().object(next_obj_with_paddr).paddr().unwrap())
                } else {
                    ut_paddr_end
                };
                let target_is_obj_with_paddr = target < ut_paddr_end;
                while cur_paddr < target {
                    let max_size_bits = usize::try_from(cur_paddr.trailing_zeros())
                        .unwrap()
                        .min((target - cur_paddr).trailing_zeros().try_into().unwrap());
                    let mut created = false;
                    if !ut.is_device() {
                        for size_bits in (0..=max_size_bits).rev() {
                            let obj_id = &mut by_size_start[size_bits];
                            // Skip embedded frames
                            while *obj_id < by_size_end[size_bits] {
                                if let Object::Frame(obj) = self.spec().object(*obj_id) {
                                    if let FrameInit::Embedded(embedded) = &obj.init {
                                        self.take_cap_for_embedded_frame(
                                            *obj_id,
                                            &embedded.get_embedded_frame(
                                                self.spec_with_sources.embedded_frame_source,
                                            ),
                                        )?;
                                        *obj_id += 1;
                                        continue;
                                    }
                                }
                                break;
                            }
                            // Create a largest possible object that would fit
                            if *obj_id < by_size_end[size_bits] {
                                let named_obj = &self.spec().named_object(*obj_id);
                                let blueprint = named_obj.object.blueprint().unwrap();
                                assert_eq!(blueprint.physical_size_bits(), size_bits);
                                trace!(
                                    "Creating kernel object: paddr=0x{:x}, size_bits={} name={:?}",
                                    cur_paddr,
                                    blueprint.physical_size_bits(),
                                    self.object_name(&named_obj.name).unwrap_or("<none>")
                                );
                                self.ut_cap(*i_ut).untyped_retype(
                                    &blueprint,
                                    &init_thread_cnode_relative_cptr(),
                                    self.alloc_orig_cslot(*obj_id).index(),
                                    1,
                                )?;
                                cur_paddr += 1 << size_bits;
                                *obj_id += 1;
                                created = true;
                                break;
                            }
                        }
                    }
                    if !created {
                        if target_is_obj_with_paddr {
                            let hold_slot = hold_slots.get_slot()?;
                            trace!(
                                "Creating dummy: paddr=0x{:x}, size_bits={}",
                                cur_paddr,
                                max_size_bits
                            );
                            self.ut_cap(*i_ut).untyped_retype(
                                &sel4::ObjectBlueprint::Untyped {
                                    size_bits: max_size_bits,
                                },
                                &init_thread_cnode_relative_cptr(),
                                hold_slot.index(),
                                1,
                            )?;
                            hold_slots.report_used();
                            cur_paddr += 1 << max_size_bits;
                        } else {
                            cur_paddr = target;
                        }
                    }
                }
                if target_is_obj_with_paddr {
                    let obj_id = next_obj_with_paddr;
                    let named_obj = &self.spec().named_object(obj_id);
                    let blueprint = named_obj.object.blueprint().unwrap();
                    trace!(
                        "Creating device object: paddr=0x{:x}, size_bits={} name={:?}",
                        cur_paddr,
                        blueprint.physical_size_bits(),
                        self.object_name(&named_obj.name).unwrap_or("<none>")
                    );
                    self.ut_cap(*i_ut).untyped_retype(
                        &blueprint,
                        &init_thread_cnode_relative_cptr(),
                        self.alloc_orig_cslot(obj_id).index(),
                        1,
                    )?;
                    cur_paddr += 1 << blueprint.physical_size_bits();
                    next_obj_with_paddr += 1;
                } else {
                    break;
                }
            }
        }

        // Ensure that we've created every root object
        for bits in 0..sel4::WORD_SIZE {
            assert_eq!(by_size_start[bits], by_size_end[bits], "!!! {}", bits);
        }

        // Create child objects

        for cover in self.spec().untyped_covers.iter() {
            let parent_obj_id = cover.parent;
            let parent = self.spec().named_object(parent_obj_id);
            let parent_cptr = self.orig_cap::<cap_type::Untyped>(parent_obj_id);
            for child_obj_id in cover.children.clone() {
                let child = &self.spec().objects[child_obj_id];
                trace!(
                    "Creating kernel object: name={:?} from {:?}",
                    self.object_name(&child.name).unwrap_or("<none>"),
                    self.object_name(&parent.name).unwrap_or("<none>"),
                );
                parent_cptr.untyped_retype(
                    &child.object.blueprint().unwrap(),
                    &init_thread_cnode_relative_cptr(),
                    self.alloc_orig_cslot(child_obj_id).index(),
                    1,
                )?;
            }
        }

        // Actually make the ASID pools. With the help of parse-capDL, we do
        // this in order of obj.asid_high, for verification reasons (see
        // upstream C CapDL loader).
        {
            for obj_id in self.spec().asid_slots.iter() {
                let ut = self.orig_cap(*obj_id);
                let slot = self.cslot_alloc_or_panic();
                init_thread::slot::ASID_CONTROL
                    .cap()
                    .asid_control_make_pool(ut, &cslot_to_relative_cptr(slot))?;
                self.set_orig_cslot(*obj_id, slot);
            }
        }

        // Create IrqHandler caps
        {
            for IrqEntry { irq, handler } in self.spec().irqs.iter() {
                let slot = self.cslot_alloc_or_panic();
                sel4::sel4_cfg_wrap_match! {
                    match self.spec().object(*handler) {
                        Object::Irq(_) => {
                            init_thread::slot::IRQ_CONTROL.cap()
                                .irq_control_get(*irq, &cslot_to_relative_cptr(slot))?;
                        }
                        #[sel4_cfg(any(ARCH_AARCH64, ARCH_AARCH32))]
                        Object::ArmIrq(obj) => {
                            sel4::sel4_cfg_if! {
                                if #[sel4_cfg(MAX_NUM_NODES = "1")] {
                                    init_thread::slot::IRQ_CONTROL.cap().irq_control_get_trigger(
                                        *irq,
                                        obj.extra.trigger,
                                        &cslot_to_relative_cptr(slot),
                                    )?;
                                } else {
                                    init_thread::slot::IRQ_CONTROL.cap().irq_control_get_trigger_core(
                                        *irq,
                                        obj.extra.trigger,
                                        obj.extra.target,
                                        &cslot_to_relative_cptr(slot),
                                    )?;
                                }
                            }
                        }
                        #[sel4_cfg(ARCH_X86_64)]
                        Object::IrqMsi(obj) => {
                            init_thread::slot::IRQ_CONTROL.cap().irq_control_get_msi(
                                obj.extra.pci_bus,
                                obj.extra.pci_dev,
                                obj.extra.pci_func,
                                obj.extra.handle,
                                *irq,
                                &cslot_to_relative_cptr(slot),
                            )?;
                        }
                        #[sel4_cfg(ARCH_X86_64)]
                        Object::IrqIOApic(obj) => {
                            init_thread::slot::IRQ_CONTROL.cap().irq_control_get_ioapic(
                                obj.extra.ioapic,
                                obj.extra.pin,
                                obj.extra.level,
                                obj.extra.polarity,
                                *irq,
                                &cslot_to_relative_cptr(slot),
                            )?;
                        }
                        _ => {
                            panic!();
                        }
                    }
                }
                self.set_orig_cslot(*handler, slot);
            }
        }

        // Create IOPort caps
        sel4::sel4_cfg_if! {
            if #[sel4_cfg(ARCH_X86_64)] {
                {
                    let ioports = self
                        .spec()
                        .filter_objects::<&object::IOPorts>()
                        .map(|(obj_id, obj)| (obj_id, obj.start_port, obj.end_port));

                    for (obj_id, start_port, end_port) in ioports {
                        let slot = self.cslot_alloc_or_panic();
                        init_thread::slot::IO_PORT_CONTROL
                            .cap()
                            .ioport_control_issue(start_port, end_port, &cslot_to_relative_cptr(slot))?;
                        self.set_orig_cslot(obj_id, slot);
                    }
                }
            }
        }

        Ok(())
    }

    fn take_cap_for_embedded_frame(
        &mut self,
        obj_id: ObjectId,
        frame: &EmbeddedFrame,
    ) -> Result<()> {
        frame.check(cap_type::Granule::FRAME_OBJECT_TYPE.bytes());
        let addr = frame.ptr() as usize;
        let slot = get_user_image_frame_slot(self.bootinfo, &self.user_image_bounds, addr);
        self.set_orig_cslot(obj_id, slot.upcast());
        Ok(())
    }

    fn init_irqs(&mut self) -> Result<()> {
        debug!("Initializing IRQs");

        let irq_notifications = self
            .spec()
            .filter_objects::<&object::Irq>()
            .map(|(obj_id, obj)| (obj_id, obj.notification()));
        let arm_irq_notifications = self
            .spec()
            .filter_objects::<&object::ArmIrq>()
            .map(|(obj_id, obj)| (obj_id, obj.notification()));
        let msi_irq_notifications = self
            .spec()
            .filter_objects::<&object::IrqMsi>()
            .map(|(obj_id, obj)| (obj_id, obj.notification()));
        let ioapic_irq_notifications = self
            .spec()
            .filter_objects::<&object::IrqIOApic>()
            .map(|(obj_id, obj)| (obj_id, obj.notification()));

        let all_irq_notifications = irq_notifications
            .chain(arm_irq_notifications)
            .chain(msi_irq_notifications)
            .chain(ioapic_irq_notifications);
        for (obj_id, notification) in all_irq_notifications {
            let irq_handler = self.orig_cap::<cap_type::IrqHandler>(obj_id);
            if let Some(logical_nfn_cap) = notification {
                let nfn = match logical_nfn_cap.badge {
                    0 => self.orig_cap(logical_nfn_cap.object),
                    badge => {
                        let orig_cptr = self.orig_relative_cptr(logical_nfn_cap.object);
                        let slot = self.cslot_alloc_or_panic();
                        let cptr = cslot_to_relative_cptr(slot);
                        cptr.mint(&orig_cptr, CapRights::all(), badge)?;
                        slot.cap().downcast()
                    }
                };
                irq_handler.irq_handler_set_notification(nfn)?;
            }
        }
        Ok(())
    }

    fn init_asids(&self) -> Result<()> {
        debug!("Initializing ASIDs");
        for (obj_id, _obj) in self
            .spec()
            .filter_objects_with::<&object::PageTable>(|obj| obj.is_root)
        {
            let pgd = self.orig_cap::<cap_type::VSpace>(obj_id);
            init_thread::slot::ASID_POOL.cap().asid_pool_assign(pgd)?;
        }
        Ok(())
    }

    fn init_frames(&mut self) -> Result<()> {
        debug!("Initializing Frames");
        for (obj_id, obj) in self.spec().filter_objects::<&object::Frame<'a, D, M>>() {
            // TODO make more platform-agnostic
            if let Some(fill) = obj.init.as_fill() {
                let entries = &fill.entries;
                if !entries.is_empty() {
                    let frame_object_type =
                        sel4::FrameObjectType::from_bits(obj.size_bits).unwrap();
                    let frame = self.orig_cap::<cap_type::UnspecifiedFrame>(obj_id);
                    self.fill_frame(frame, frame_object_type, entries)?;
                }
            }
        }
        Ok(())
    }

    fn fill_frame(
        &self,
        frame: sel4::Cap<cap_type::UnspecifiedFrame>,
        frame_object_type: sel4::FrameObjectType,
        fill: &[FillEntry<D>],
    ) -> Result<()> {
        frame.frame_map(
            init_thread::slot::VSPACE.cap(),
            self.copy_addrs.select(frame_object_type),
            CapRights::read_write(),
            vm_attributes_from_whether_cached(false),
        )?;
        for entry in fill.iter() {
            let offset = entry.range.start;
            let length = entry.range.end - entry.range.start;
            assert!(entry.range.end <= frame_object_type.bytes());
            let dst_frame = self.copy_addrs.select(frame_object_type) as *mut u8;
            let dst = unsafe { slice::from_raw_parts_mut(dst_frame.add(offset), length) };
            match &entry.content {
                FillEntryContent::Data(content_data) => {
                    content_data.copy_out(self.spec_with_sources.content_source, dst);
                }
                FillEntryContent::BootInfo(content_bootinfo) => {
                    for extra in self.bootinfo.extra() {
                        if extra.id == (&content_bootinfo.id).into() {
                            let n = dst.len().min(
                                extra
                                    .content_with_header()
                                    .len()
                                    .saturating_sub(content_bootinfo.offset),
                            );
                            if n > 0 {
                                dst[..n].copy_from_slice(
                                    &extra.content_with_header()
                                        [content_bootinfo.offset..(content_bootinfo.offset + n)],
                                );
                            }
                        }
                    }
                }
            }
        }
        frame.frame_unmap()?;
        Ok(())
    }

    fn init_vspaces(&mut self) -> Result<()> {
        debug!("Initializing VSpaces");
        for (obj_id, obj) in self
            .spec()
            .filter_objects_with::<&object::PageTable>(|obj| obj.is_root)
        {
            let vspace = self.orig_cap::<cap_type::VSpace>(obj_id);
            self.init_vspace(vspace, 0, 0, obj)?;
        }
        Ok(())
    }

    fn init_vspace(
        &mut self,
        vspace: sel4::cap::VSpace,
        level: usize,
        vaddr: usize,
        obj: &object::PageTable,
    ) -> Result<()> {
        for (i, entry) in obj.entries() {
            let vaddr = vaddr + (i << sel4::vspace_levels::step_bits(level));
            match entry {
                PageTableEntry::Frame(cap) => {
                    let frame = self.orig_cap::<cap_type::UnspecifiedFrame>(cap.object);
                    let rights = (&cap.rights).into();
                    self.copy(frame)?
                        .frame_map(vspace, vaddr, rights, cap.vm_attributes())?;
                }
                PageTableEntry::PageTable(cap) => {
                    self.orig_cap::<cap_type::UnspecifiedIntermediateTranslationTable>(cap.object)
                        .generic_intermediate_translation_table_map(
                            sel4::TranslationTableObjectType::from_level(level + 1).unwrap(),
                            vspace,
                            vaddr,
                            cap.vm_attributes(),
                        )?;
                    let obj = self
                        .spec()
                        .lookup_object::<&object::PageTable>(cap.object)?;
                    self.init_vspace(vspace, level + 1, vaddr, obj)?;
                }
            }
        }
        Ok(())
    }

    #[sel4::sel4_cfg(KERNEL_MCS)]
    fn init_sched_contexts(&self) -> Result<()> {
        debug!("Initializing scheduling contexts");
        for (obj_id, _obj) in self.spec().filter_objects::<&object::SchedContext>() {
            self.init_sched_context(obj_id, 0)?;
        }
        Ok(())
    }

    #[sel4::sel4_cfg(KERNEL_MCS)]
    fn init_sched_context(&self, obj_id: ObjectId, affinity: usize) -> Result<()> {
        let obj = self.spec().lookup_object::<&object::SchedContext>(obj_id)?;
        let sched_context = self.orig_cap::<cap_type::SchedContext>(obj_id);
        self.bootinfo
            .sched_control()
            .index(affinity)
            .cap()
            .sched_control_configure_flags(
                sched_context,
                obj.extra.budget,
                obj.extra.period,
                0,
                obj.extra.badge,
                0,
            )?;
        Ok(())
    }

    fn init_tcbs(&mut self) -> Result<()> {
        debug!("Initializing TCBs");

        for (obj_id, obj) in self.spec().filter_objects::<&object::Tcb>() {
            let tcb = self.orig_cap::<cap_type::Tcb>(obj_id);

            if let Some(bound_notification) = obj.bound_notification() {
                let bound_notification =
                    self.orig_cap::<cap_type::Notification>(bound_notification.object);
                tcb.tcb_bind_notification(bound_notification)?;
            }

            sel4::sel4_cfg_if! {
                if #[sel4_cfg(all(ARCH_AARCH64, ARM_HYPERVISOR_SUPPORT))] {
                    if let Some(vcpu) = obj.vcpu() {
                        let vcpu = self.orig_cap::<cap_type::VCpu>(vcpu.object);
                        vcpu.vcpu_set_tcb(tcb)?;
                    }
                }
            }

            {
                let cspace = self.orig_cap(obj.cspace().object);
                let cspace_root_data = sel4::CNodeCapData::new(
                    obj.cspace().guard,
                    obj.cspace().guard_size.try_into().unwrap(),
                );
                let vspace = self.orig_cap(obj.vspace().object);
                let ipc_buffer_addr = obj.extra.ipc_buffer_addr;
                let ipc_buffer_frame = self.orig_cap(obj.ipc_buffer().object);

                let authority = init_thread::slot::TCB.cap();
                let max_prio = obj.extra.max_prio.into();
                let prio = obj.extra.prio.into();

                #[allow(unused_variables)]
                let affinity: usize = obj.extra.affinity.try_into()?;

                sel4::sel4_cfg_if! {
                    if #[sel4_cfg(KERNEL_MCS)] {
                        if let Some(sched_context_cap) = obj.sc() {
                            self.init_sched_context(sched_context_cap.object, affinity)?;
                        }

                        tcb.tcb_configure(
                            cspace,
                            cspace_root_data,
                            vspace,
                            ipc_buffer_addr,
                            ipc_buffer_frame,
                        )?;

                        let sc = match obj.sc() {
                            None => init_thread::slot::NULL.cap().cast::<cap_type::SchedContext>(),
                            Some(cap) => self.orig_cap::<cap_type::SchedContext>(cap.object),
                        };

                        let fault_ep = match obj.mcs_fault_ep() {
                            None => init_thread::slot::NULL.cap().cast::<cap_type::Endpoint>(),
                            Some(cap) => {
                                let orig = self.orig_cap::<cap_type::Endpoint>(cap.object);
                                let badge = cap.badge;
                                let rights = (&cap.rights).into();
                                if badge == 0 && rights == CapRights::all() {
                                    orig
                                } else {
                                    let src = init_thread::slot::CNODE.cap().relative(orig);
                                    let new = self.cslot_alloc_or_panic().cap();
                                    let dst = init_thread::slot::CNODE.cap().relative(new);
                                    dst.mint(&src, rights, badge)?;
                                    new.cast()
                                }
                            },
                        };

                        let temp_fault_ep = match obj.temp_fault_ep() {
                            None => init_thread::slot::NULL.cap().cast::<cap_type::Endpoint>(),
                            Some(cap) => {
                                assert_eq!(cap.badge, 0); // HACK
                                self.orig_cap::<cap_type::Endpoint>(cap.object)
                            },
                        };

                        tcb.tcb_set_sched_params(
                            authority,
                            max_prio,
                            prio,
                            sc,
                            fault_ep,
                        )?;

                        tcb.tcb_set_timeout_endpoint(temp_fault_ep)?;
                    } else {
                        let fault_ep = sel4::CPtr::from_bits(obj.extra.master_fault_ep.unwrap());

                        tcb.tcb_configure(
                            fault_ep,
                            cspace,
                            cspace_root_data,
                            vspace,
                            ipc_buffer_addr,
                            ipc_buffer_frame,
                        )?;

                        tcb.tcb_set_sched_params(
                            authority,
                            max_prio,
                            prio,
                        )?;

                        sel4::sel4_cfg_if! {
                            if #[sel4_cfg(not(MAX_NUM_NODES = "1"))] {
                                tcb.tcb_set_affinity(affinity.try_into().unwrap())?;
                            }
                        }
                    }
                }
            }

            {
                let mut regs = sel4::UserContext::default();
                *regs.generic_pc_mut() = obj.extra.ip;
                *regs.generic_sp_mut() = obj.extra.sp;
                for (i, value) in obj.extra.gprs.iter().enumerate() {
                    *regs.c_param_mut(i) = *value;
                }
                tcb.tcb_write_all_registers(false, &mut regs)?;
            }

            sel4::sel4_cfg_if! {
                if #[sel4_cfg(DEBUG_BUILD)] {
                    if let Some(name) = self.object_name(self.spec().name(obj_id)) {
                        tcb.debug_name(name.as_bytes());
                    }
                }
            }
        }
        Ok(())
    }

    fn init_cspaces(&self) -> Result<()> {
        debug!("Initializing CSpaces");

        for (obj_id, obj) in self.spec().filter_objects::<&object::CNode>() {
            let cnode = self.orig_cap::<cap_type::CNode>(obj_id);
            for (i, cap) in obj.slots() {
                let badge = cap.badge();
                let rights = cap.rights().map(From::from).unwrap_or(CapRights::all());
                let src = init_thread::slot::CNODE
                    .cap()
                    .relative(self.orig_cap::<cap_type::Unspecified>(cap.obj()));
                let dst = cnode.relative_bits_with_depth((*i).try_into().unwrap(), obj.size_bits);
                match badge {
                    None => dst.copy(&src, rights),
                    Some(badge) => dst.mint(&src, rights, badge),
                }?;
            }
        }
        Ok(())
    }

    fn start_threads(&self) -> Result<()> {
        debug!("Starting threads");
        for (obj_id, obj) in self.spec().filter_objects::<&object::Tcb>() {
            let tcb = self.orig_cap::<cap_type::Tcb>(obj_id);
            if obj.extra.resume {
                tcb.tcb_resume()?;
            }
        }
        Ok(())
    }

    //

    fn copy<U: sel4::CapType>(&mut self, cap: sel4::Cap<U>) -> Result<sel4::Cap<U>> {
        let slot = self.cslot_alloc_or_panic();
        let src = init_thread::slot::CNODE.cap().relative(cap);
        cslot_to_relative_cptr(slot).copy(&src, CapRights::all())?;
        Ok(slot.cap().downcast())
    }

    //

    fn cslot_alloc_or_panic(&mut self) -> Slot {
        self.cslot_allocator.alloc_or_panic()
    }

    fn set_orig_cslot(&mut self, obj_id: ObjectId, slot: Slot) {
        self.buffers.per_obj_mut()[obj_id].orig_slot = Some(slot);
    }

    fn orig_cslot(&self, obj_id: ObjectId) -> Slot {
        self.buffers.per_obj()[obj_id].orig_slot.unwrap()
    }

    fn alloc_orig_cslot(&mut self, obj_id: ObjectId) -> Slot {
        let slot = self.cslot_alloc_or_panic();
        self.set_orig_cslot(obj_id, slot);
        slot
    }

    fn orig_cap<U: sel4::CapType>(&self, obj_id: ObjectId) -> sel4::Cap<U> {
        self.orig_cslot(obj_id).cap().downcast()
    }

    fn orig_relative_cptr(&self, obj_id: ObjectId) -> sel4::AbsoluteCPtr {
        cslot_to_relative_cptr(self.orig_cslot(obj_id))
    }

    fn ut_cap(&self, ut_index: usize) -> sel4::cap::Untyped {
        self.bootinfo.untyped().index(ut_index).cap()
    }
}

fn cslot_to_relative_cptr(slot: Slot) -> sel4::AbsoluteCPtr {
    init_thread::slot::CNODE.cap().relative(slot.cptr())
}

fn init_thread_cnode_relative_cptr() -> sel4::AbsoluteCPtr {
    init_thread::slot::CNODE.cap().relative_self()
}
