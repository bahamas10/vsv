/*
 * Author: Dave Eddy <dave@daveeddy.com>
 * Date: January 26, 2022
 * License: MIT
 */

/*!
 * Contains the `die!()` convenience macro for exiting a program with a code and
 * message.
 */

/**
 * Exit the current program with a code and optional message.
 *
 * # Usage
 *
 * Exit the program successfully with no message:
 *
 * ```
 * die!(0);
 * ```
 *
 * Exit the program with code 1 and a message:
 *
 * ```
 * die!(1, "uh oh");
 * ```
 *
 * Exit the program with code 57 and an error message:
 *
 * ```
 * let bad_num: u32 = "foo".parse();
 *
 * if Err(err) = bad_num {
 *     die!(57, "number parsing failed: {:?}", err);
 * }
 * ```
 */
macro_rules! die {
    () => {
        ::std::process::exit(1);
    };

    ($code:expr $(,)?) => {
        ::std::process::exit($code);
    };

    ($code:expr, $fmt:expr $(, $args:expr )* $(,)?) => {{
        eprintln!($fmt $( , $args )*);
        ::std::process::exit($code);
    }};
}

pub(crate) use die;
