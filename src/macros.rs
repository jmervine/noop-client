// -- macros
#[macro_export]
macro_rules! is_verbose {
    () => { ( unsafe { crate::VERBOSE } ) }
}

#[macro_export]
macro_rules! set_verbose {
    ($v:expr) => { unsafe {
        crate::VERBOSE = $v
    } }
}

#[macro_export]
macro_rules! split_err {
    ($e:expr) => { Err(ConfigError::ToHeaderSplitError($e)) };
}

#[macro_export]
macro_rules! debug {
    ($s:expr) => {
        if is_verbose!() {
            println!("[DEBUG]: {}", $s)
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
macro_rules! echeck {
    ($v:expr) => {
        if $v.is_err() {
            eprintln!("{:?} at {}:{}", $v, file!(), line!());
            std::process::exit(1);
        }
    };
}

// TODO: Remove macros after fixing error handling to "be better"(^TM)
#[macro_export]
macro_rules! bad_error {
    ($msg:expr) => {
        panic!(
            "---\n\
            ERROR: {:?}\n\
            >TODO: Rework the entire function to allow for better error handling @ {}:{}.\n",
            $msg, file!(), line!()
        );
    }
}

// TODO: Remove macros after fixing error handling to "be better"(^TM)
#[macro_export]
macro_rules! some_bad_error {
    ($err:expr) => {
        if let Some(err) = $err {
            bad_error!(err);
        }
    }
}

mod test {
    #[test]
    fn is_verbose_test() {
        unsafe { crate::VERBOSE = false; }
        assert!(!is_verbose!(), "'is_verbose()' should return false");

        unsafe { crate::VERBOSE = true; }
        assert!(is_verbose!(), "'is_verbose()' should return true");
    }

    #[test]
    fn set_verbose_test() {
        unsafe { crate::VERBOSE = false; }
        set_verbose!(true);
        assert!(unsafe { crate::VERBOSE }, "should return true");

        unsafe { crate::VERBOSE = true; }
        set_verbose!(false);
        assert!(!unsafe { crate::VERBOSE }, "should return false");
    }
}