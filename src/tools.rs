use crate::constants as consts;

pub(crate) fn debug_log(log_message: &str) {
    if consts::DEBUG {
        eprintln!("{}", log_message);
    }
}