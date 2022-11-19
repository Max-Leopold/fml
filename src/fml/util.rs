#[macro_export]
macro_rules! skip_err {
    ($res:expr) => {
        match $res {
            Ok(val) => val,
            Err(e) => {
                log::info!("Error occured: {}; Skipping.", e);
                continue;
            }
        }
    };
}

#[macro_export]
macro_rules! skip_none {
    ($res:expr) => {
        match $res {
            Some(val) => val,
            None => {
                log::info!("Skipping None value.");
                continue;
            }
        }
    };
}
