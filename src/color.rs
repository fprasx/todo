use std::io::Write;

pub struct Printer();

macro_rules! printer_impl {
    ($($color:ident=$color_str:expr)*) => {
        $(
            pub fn $color(self, s: impl std::fmt::Display) -> Self {
                print!("{}{s}\x1b[0m", $color_str);
                self
            }
        )*
    };
}

impl Printer {
    /// Make sure all changes are emitted
    pub fn finish(self) -> Self {
        let _ = std::io::stdout().flush();
        self
    }

    /// Make sure all changes are emitted and append newline to end
    pub fn finish_nl(self) -> Self {
        let _ = std::io::stdout().flush();
        println!();
        self
    }

    pub fn default(self, s: impl std::fmt::Display) -> Self {
        print!("{s}");
        self
    }

    printer_impl!(
        black = "\x1b[0;30m"
        red = "\x1b[0;31m"
        green = "\x1b[0;32m"
        yellow = "\x1b[0;33m"
        blue = "\x1b[0;34m"
        purple = "\x1b[0;35m"
        cyan = "\x1b[0;36m"
        white = "\x1b[0;37m"
        bblack = "\x1b[1;30m"
        bred = "\x1b[1;31m"
        bgreen = "\x1b[1;32m"
        byellow = "\x1b[1;33m"
        bblue = "\x1b[1;34m"
        bpurple = "\x1b[1;35m"
        bcyan = "\x1b[1;36m"
        bwhite = "\x1b[1;37m"
        ublack = "\x1b[4;30m"
        ured = "\x1b[4;31m"
        ugreen = "\x1b[4;32m"
        uyellow = "\x1b[4;33m"
        ublue = "\x1b[4;34m"
        upurple = "\x1b[4;35m"
        ucyan = "\x1b[4;36m"
        uwhite = "\x1b[4;37m"
        on_black = "\x1b[40m"
        on_red = "\x1b[41m"
        on_green = "\x1b[42m"
        on_yellow = "\x1b[43m"
        on_blue = "\x1b[44m"
        on_purple = "\x1b[45m"
        on_cyan = "\x1b[46m"
        on_white = "\x1b[47m"
        iblack = "\x1b[0;90m"
        ired = "\x1b[0;91m"
        igreen = "\x1b[0;92m"
        iyellow = "\x1b[0;93m"
        iblue = "\x1b[0;94m"
        ipurple = "\x1b[0;95m"
        icyan = "\x1b[0;96m"
        iwhite = "\x1b[0;97m"
        biblack = "\x1b[1;90m"
        bired = "\x1b[1;91m"
        bigreen = "\x1b[1;92m"
        biyellow = "\x1b[1;93m"
        biblue = "\x1b[1;94m"
        bipurple = "\x1b[1;95m"
        bicyan = "\x1b[1;96m"
        biwhite = "\x1b[1;97m"
        on_iblack = "\x1b[0;100m"
        on_ired = "\x1b[0;101m"
        on_igreen = "\x1b[0;102m"
        on_iyellow = "\x1b[0;103m"
        on_iblue = "\x1b[0;104m"
        on_ipurple = "\x1b[0;105m"
        on_icyan = "\x1b[0;106m"
    );
}

// Reset
pub const RESET: &str = "\x1b[0m"; // Text Reset

// Regular Colors
pub const BLACK: &str = "\x1b[0;30m"; // Black
pub const RED: &str = "\x1b[0;31m"; // Red
pub const GREEN: &str = "\x1b[0;32m"; // Green
pub const YELLOW: &str = "\x1b[0;33m"; // Yellow
pub const BLUE: &str = "\x1b[0;34m"; // Blue
pub const PURPLE: &str = "\x1b[0;35m"; // Purple
pub const CYAN: &str = "\x1b[0;36m"; // Cyan
pub const WHITE: &str = "\x1b[0;37m"; // White

// Bold
pub const BBLACK: &str = "\x1b[1;30m"; // Black
pub const BRED: &str = "\x1b[1;31m"; // Red
pub const BGREEN: &str = "\x1b[1;32m"; // Green
pub const BYELLOW: &str = "\x1b[1;33m"; // Yellow
pub const BBLUE: &str = "\x1b[1;34m"; // Blue
pub const BPURPLE: &str = "\x1b[1;35m"; // Purple
pub const BCYAN: &str = "\x1b[1;36m"; // Cyan
pub const BWHITE: &str = "\x1b[1;37m"; // White

// Underline
pub const UBLACK: &str = "\x1b[4;30m"; // Black
pub const URED: &str = "\x1b[4;31m"; // Red
pub const UGREEN: &str = "\x1b[4;32m"; // Green
pub const UYELLOW: &str = "\x1b[4;33m"; // Yellow
pub const UBLUE: &str = "\x1b[4;34m"; // Blue
pub const UPURPLE: &str = "\x1b[4;35m"; // Purple
pub const UCYAN: &str = "\x1b[4;36m"; // Cyan
pub const UWHITE: &str = "\x1b[4;37m"; // White

// Background
pub const ON_BLACK: &str = "\x1b[40m"; // Black
pub const ON_RED: &str = "\x1b[41m"; // Red
pub const ON_GREEN: &str = "\x1b[42m"; // Green
pub const ON_YELLOW: &str = "\x1b[43m"; // Yellow
pub const ON_BLUE: &str = "\x1b[44m"; // Blue
pub const ON_PURPLE: &str = "\x1b[45m"; // Purple
pub const ON_CYAN: &str = "\x1b[46m"; // Cyan
pub const ON_WHITE: &str = "\x1b[47m"; // White

// High Intensity
pub const IBLACK: &str = "\x1b[0;90m"; // Black
pub const IRED: &str = "\x1b[0;91m"; // Red
pub const IGREEN: &str = "\x1b[0;92m"; // Green
pub const IYELLOW: &str = "\x1b[0;93m"; // Yellow
pub const IBLUE: &str = "\x1b[0;94m"; // Blue
pub const IPURPLE: &str = "\x1b[0;95m"; // Purple
pub const ICYAN: &str = "\x1b[0;96m"; // Cyan
pub const IWHITE: &str = "\x1b[0;97m"; // White

// Bold High Intensity
pub const BIBLACK: &str = "\x1b[1;90m"; // Black
pub const BIRED: &str = "\x1b[1;91m"; // Red
pub const BIGREEN: &str = "\x1b[1;92m"; // Green
pub const BIYELLOW: &str = "\x1b[1;93m"; // Yellow
pub const BIBLUE: &str = "\x1b[1;94m"; // Blue
pub const BIPURPLE: &str = "\x1b[1;95m"; // Purple
pub const BICYAN: &str = "\x1b[1;96m"; // Cyan
pub const BIWHITE: &str = "\x1b[1;97m"; // White

// High Intensity backgrounds
pub const ON_IBLACK: &str = "\x1b[0;100m"; // Black
pub const ON_IRED: &str = "\x1b[0;101m"; // Red
pub const ON_IGREEN: &str = "\x1b[0;102m"; // Green
pub const ON_IYELLOW: &str = "\x1b[0;103m"; // Yellow
pub const ON_IBLUE: &str = "\x1b[0;104m"; // Blue
pub const ON_IPURPLE: &str = "\x1b[0;105m"; // Purple
pub const ON_ICYAN: &str = "\x1b[0;106m"; // Cyan
