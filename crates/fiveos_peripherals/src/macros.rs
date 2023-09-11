#[macro_export]
macro_rules! print {
    ($writee:ident, $($args:tt)+) => ({
        use core::fmt::Write;
        // use fiveos_virtio::uart::{Uart0};
        // let mut uart0 = unsafe {Uart0::new()};
        let _ = write!($writee, $($args)+);
    });
}

#[macro_export]
macro_rules! println
{
	($writee:ident,) => ({
		print!($writee, "\r\n")
	});
	($writee:ident, $fmt:expr) => ({
		print!($writee, concat!($fmt, "\r\n"))
	});
	($writee:ident, $fmt:expr, $($args:tt)+) => ({
		print!($writee, concat!($fmt, "\r\n"), $($args)+)
	});
}

#[macro_export]
macro_rules! printhdr {
    ($writee:ident,) => {{
        let len = 0;
        const HEADER_WIDTH: usize = 64;
        let remainder = (HEADER_WIDTH - len);
        for _ in 0..remainder {
            print!($writee, "~");
        }
        println!($writee,);
    }};
    ($writee:ident,$fmt:expr) => {{
        let len = $fmt.len() + 2;
        const HEADER_WIDTH: usize = 64;
        let remainder = (HEADER_WIDTH - len) / 2;
        for _ in 0..remainder {
            print!($writee, "~");
        }
        print!($writee, " ");
        print!($writee, $fmt);
        print!($writee, " ");
        for _ in 0..remainder {
            print!($writee, "~");
        }
        if remainder * 2 + len != HEADER_WIDTH {
            print!($writee, "~");
        }
        println!($writee,);
    }};
}

#[macro_export]
macro_rules! print_title {
    ($writee:ident,$fmt:expr) => {{
        let len = $fmt.len() + 2;
        const HEADER_WIDTH: usize = 64;
        let remainder = (HEADER_WIDTH - len) / 2;

        for _ in 0..HEADER_WIDTH {
            print!($writee, "#");
        }
        println!($writee,);
        print!($writee, "#");

        for _ in 0..remainder {
            print!($writee, " ");
        }

        print!($writee, $fmt);

        for _ in 0..remainder {
            print!($writee, " ");
        }
        if remainder * 2 + len != HEADER_WIDTH {
            print!($writee, " ");
        }
        print!($writee, "#");
        println!($writee,);
        for _ in 0..HEADER_WIDTH {
            print!($writee, "#");
        }
        println!($writee,);
    }};
}
