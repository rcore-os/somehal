macro_rules! bail {
    ($e:expr) => {
        match $e {
            Status::SUCCESS => {}
            err => return err,
        }
    };
}