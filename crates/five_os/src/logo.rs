use core::fmt::Write;

use crate::{print, println};

const LOGO: &str = r"
                    _________   ______  ____  ____
                   / __/  _/ | / / __/ / __ \/ __/
                  / _/_/ / | |/ / _/  / /_/ /\ \
                 /_/ /___/ |___/___/  \____/___/
";

pub fn print_logo(writee: &mut impl Write) {
    println!(writee, "{}", LOGO);
}
