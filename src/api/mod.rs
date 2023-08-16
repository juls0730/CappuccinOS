#[macro_export]
macro_rules! entry_point {
    ($path:path) => {
        #[panic_handler]
        fn panic(_info: &core::panic::PanicInfo) -> ! {
            $crate::api::syscall::write("An exception occured!\n");
            loop {}
        }
    };
}

pub mod syscall;
