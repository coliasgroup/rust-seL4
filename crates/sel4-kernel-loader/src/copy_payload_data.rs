use core::ptr;
use core::slice;

use sel4_kernel_loader_payload_types::{Region, RegionContent};

pub(crate) fn copy_payload_data<T: RegionContent>(
    regions: &[Region<usize, T>],
    region_content_source: &T::Source,
) {
    for region in regions.iter() {
        let dst = unsafe {
            slice::from_raw_parts_mut(
                ptr::from_exposed_addr_mut(region.phys_addr_range.start.try_into().unwrap()),
                (region.phys_addr_range.end - region.phys_addr_range.start)
                    .try_into()
                    .unwrap(),
            )
        };
        match &region.content {
            Some(src) => {
                src.copy_out(region_content_source, dst);
            }
            None => {
                // NOTE slice::fill is too slow
                unsafe {
                    ptr::write_bytes(dst.as_mut_ptr(), 0, dst.len());
                }
            }
        }
    }
}
