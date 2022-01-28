/*
 * A simple macro to kill a process with a specified exit code.
 *
 * Author: Dave Eddy <dave@daveeddy.com>
 * Date: January 26, 2022
 * License: MIT
 */

/*
 * Usage:
 *
 * die!()   // exit with code 1
 * die!(24) // exit with code 24
 * die!(12, "uh oh: {}", err); // print the message to stderr and exit with code 12
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
