// -- common utils
// if needed
#[derive(Debug, Clone, PartialEq)]
pub enum Errors {
    Error(String),
    Ignorable,
    // Fatal(T),
    //Warn(String),
}

// Implement the std::fmt::Display trait to provide a human-readable description of the error
impl std::fmt::Display for Errors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Errors::Error(inner) => write!(f, "Error[{}:{}] {}", file!(), line!(), inner),
            Errors::Ignorable => write!(f, "Ignorable error."),
            // Add formatting for other error variants as needed
        }
    }
}

// -- macros
pub(crate) mod macros {
    #[macro_export]
    macro_rules! exit {
        ($m:expr) => {{
            if !$m.is_empty() {
                println!("{}", $m);
            }
            std::process::exit(0)
        }};
    }

    #[macro_export]
    macro_rules! error {
        ($m:expr) => {{
            Err(utils::Errors::Error($m.unwrap_err().to_string()))
        }};
    }

    #[macro_export]
    macro_rules! error_str {
        ($m:expr) => {{
            Err(utils::Errors::Error(format!("{:?}", $m)))
        }};
    }

    #[macro_export]
    macro_rules! is_verbose {
        () => {
            (unsafe { crate::VERBOSE })
        };
    }

    #[macro_export]
    macro_rules! set_verbose {
        ($v:expr) => {
            unsafe { crate::VERBOSE = $v }
        };
    }

    #[macro_export]
    macro_rules! debug {
        ($s:expr) => {
            if is_verbose!() {
                println!("Debug[{}:{}] {}", file!(), line!(), $s)
            }
        };
    }

    #[macro_export]
    macro_rules! debugln {
        ($s:expr) => {
            debug!(format!("{}\n", $s))
        };
    }

    #[macro_export]
    macro_rules! unless_debug {
        ($s:expr) => {
            if !is_verbose!() {
                print!("{}", $s)
            }
        };
    }

    #[macro_export]
    macro_rules! unless_debugln {
        ($s:expr) => {
            unless_debug!(format!("{}\n", $s))
        };
    }

    #[macro_export]
    macro_rules! response_error_vector {
        ($str:expr) => {{
            let ret: Vec<Result<reqwest::Response, utils::Errors>> =
                vec![Err(utils::Errors::Error($str))];
            ret
        }};
    }

    mod test {
        #[test]
        fn is_verbose_test() {
            unsafe {
                crate::VERBOSE = false;
            }
            assert!(!is_verbose!(), "'is_verbose()' should return false");

            unsafe {
                crate::VERBOSE = true;
            }
            assert!(is_verbose!(), "'is_verbose()' should return true");
        }

        #[test]
        fn set_verbose_test() {
            unsafe {
                crate::VERBOSE = false;
            }
            set_verbose!(true);
            assert!(unsafe { crate::VERBOSE }, "should return true");

            unsafe {
                crate::VERBOSE = true;
            }
            set_verbose!(false);
            assert!(!unsafe { crate::VERBOSE }, "should return false");
        }
    }
}
