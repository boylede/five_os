#[macro_export]
macro_rules! print {
    ($f:ident, $($args:tt)+) => ({
        use core::fmt::Write;
        let _ = write!($f, $($args)+);
    });
}

#[macro_export]
macro_rules! println
{
	($f:ident,) => ({
		print!($f, "\r\n")
	});
	($f:ident, $fmt:expr) => ({
		print!($f, concat!($fmt, "\r\n"))
	});
	($f:ident, $fmt:expr, $($args:tt)+) => ({
		print!($f, concat!($fmt, "\r\n"), $($args)+)
	});
}

#[macro_export]
macro_rules! printhdr {
    ($f:ident,) => {{
        let len = 0;
        const HEADER_WIDTH: usize = 64;
        let remainder = (HEADER_WIDTH - len);
        for _ in 0..remainder {
            print!($f, "~");
        }
        println!($f,);
    }};
    ($f:ident,$fmt:expr) => {{
        let len = $fmt.len() + 2;
        const HEADER_WIDTH: usize = 64;
        let remainder = (HEADER_WIDTH - len) / 2;
        for _ in 0..remainder {
            print!($f, "~");
        }
        print!($f, " ");
        print!($f, $fmt);
        print!($f, " ");
        for _ in 0..remainder {
            print!($f, "~");
        }
        if remainder * 2 + len != HEADER_WIDTH {
            print!($f, "~");
        }
        println!($f,);
    }};
}

#[macro_export]
macro_rules! print_title {
    ($f:ident,$fmt:expr) => {{
        let len = $fmt.len() + 2;
        const HEADER_WIDTH: usize = 64;
        let remainder = (HEADER_WIDTH - len) / 2;

        for _ in 0..HEADER_WIDTH {
            print!($f, "#");
        }
        println!($f,);
        print!($f, "#");

        for _ in 0..remainder {
            print!($f, " ");
        }

        print!($f, $fmt);

        for _ in 0..remainder {
            print!($f, " ");
        }
        if remainder * 2 + len != HEADER_WIDTH {
            print!($f, " ");
        }
        print!($f, "#");
        println!($f,);
        for _ in 0..HEADER_WIDTH {
            print!($f, "#");
        }
        println!($f,);
    }};
}
