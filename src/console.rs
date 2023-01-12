mod commands;

#[macro_export]
macro_rules! print {
    ($($args:tt)+) => ({
        use core::fmt::Write;
        use fiveos_riscv::cpu;
        use fiveos_virtio::uart::UART_BASE_ADDRESS;
        let _ = write!(cpu::uart::Uart::<{UART_BASE_ADDRESS}>::default(), $($args)+);
    });
}

#[macro_export]
macro_rules! println
{
	() => ({
		print!("\r\n")
	});
	($fmt:expr) => ({
		print!(concat!($fmt, "\r\n"))
	});
	($fmt:expr, $($args:tt)+) => ({
		print!(concat!($fmt, "\r\n"), $($args)+)
	});
}

#[macro_export]
macro_rules! printhdr {
    () => {{
        let len = 0;
        const HEADER_WIDTH: usize = 64;
        let remainder = (HEADER_WIDTH - len);
        for _ in 0..remainder {
            print!("~");
        }
        println!();
    }};
    ($fmt:expr) => {{
        let len = $fmt.len() + 2;
        const HEADER_WIDTH: usize = 64;
        let remainder = (HEADER_WIDTH - len) / 2;
        for _ in 0..remainder {
            print!("~");
        }
        print!(" ");
        print!($fmt);
        print!(" ");
        for _ in 0..remainder {
            print!("~");
        }
        if remainder * 2 + len != HEADER_WIDTH {
            print!("~");
        }
        println!();
    }};
}

#[macro_export]
macro_rules! print_title {
    ($fmt:expr) => {{
        let len = $fmt.len() + 2;
        const HEADER_WIDTH: usize = 64;
        let remainder = (HEADER_WIDTH - len) / 2;

        for _ in 0..HEADER_WIDTH {
            print!("#");
        }
        println!();
        print!("#");

        for _ in 0..remainder {
            print!(" ");
        }

        print!($fmt);

        for _ in 0..remainder {
            print!(" ");
        }
        if remainder * 2 + len != HEADER_WIDTH {
            print!(" ");
        }
        print!("#");
        println!();
        for _ in 0..HEADER_WIDTH {
            print!("#");
        }
        println!();
    }};
}
