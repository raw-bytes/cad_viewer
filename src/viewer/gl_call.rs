use glow::{
    Context, HasContext, INVALID_ENUM, INVALID_FRAMEBUFFER_OPERATION, INVALID_INDEX,
    INVALID_OPERATION, INVALID_VALUE, NO_ERROR, OUT_OF_MEMORY, STACK_OVERFLOW, STACK_UNDERFLOW,
};

use log::error;

/// Returns the corresponding error string for the given OpenGL error code
///
///* `error_code` - OpenGL error code
fn code_to_string(error_code: u32) -> &'static str {
    match error_code {
        NO_ERROR => "no error",
        INVALID_ENUM => "invalid enumerant",
        INVALID_VALUE => "invalid value",
        INVALID_OPERATION => "invalid operation",
        STACK_OVERFLOW => "stack overflow",
        STACK_UNDERFLOW => "stack underflow",
        OUT_OF_MEMORY => "out of memory",
        INVALID_FRAMEBUFFER_OPERATION => "invalid framebuffer operation",
        INVALID_INDEX => "index is invalid",
        _ => "unknown error code",
    }
}

/// Checks if the previous OpenGL function calls caused any errors.
/// If there was an error, write it into the log.
///
/// # Arguments
///
/// * `context` - The context to check for an error.
/// * `filename` - The source filename where the error was caused
/// * `line` - The line in the source filename where the error was caused
/// * `column` - The column in the source filename where the error was caused
pub fn check(context: &Context, filename: &str, line: u32, column: u32) {
    let error_code = unsafe { context.get_error() };

    if error_code != NO_ERROR {
        let error_msg = code_to_string(error_code);
        error!(
            "{} ({}:{}): Found OpenGL error '{}'",
            filename, line, column, error_msg
        );
    }
}

/// Internal use only function which performs the additional steps for an OpenGL function call.
/// Returns the passed return value t.
///
/// # Arguments
///
/// * `context` - The GLOW context for checking the error.
/// * `t` - The return value of the previously executed OpenGL function call
/// * `filename` - The source filename where the function has been executed
/// * `line` - The line in the source code file
/// * `column` - Then column in the source code file
#[inline]
pub fn gl_call_helper<T>(t: T, context: &Context, filename: &str, line: u32, column: u32) -> T {
    check(context, filename, line, column);

    t
}

/// Encapsulates an OpenGL function call and performs internal checks and OpenGL call counting
#[macro_export]
macro_rules! gl_call {
    ($ctx:ident, $function:ident, $($params:tt)*) => {
        $crate::viewer::gl_call::gl_call_helper(
            unsafe { $ctx.$function($($params)*) },
            $ctx,
            file!(),
            line!(),
            column!(),
        )
    };
}
