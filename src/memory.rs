use core::num::NonZeroUsize;

pub mod phys;

pub type PhysicalAddress = NonZeroUsize;

pub fn init() {
    phys::init();
}
