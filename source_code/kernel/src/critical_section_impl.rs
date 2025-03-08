use critical_section::RawRestoreState;

struct MyCriticalSection;

unsafe impl critical_section::Impl for MyCriticalSection {
    unsafe fn acquire() -> RawRestoreState {
        // Implement the logic to disable interrupts (if needed)
        RawRestoreState::default()
    }

    unsafe fn release(_token: RawRestoreState) {
        // Implement the logic to enable interrupts (if needed)
    }
}

critical_section::set_impl!(MyCriticalSection);
