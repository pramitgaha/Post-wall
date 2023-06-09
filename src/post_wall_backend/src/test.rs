use candid::{export_service, Principal, Nat};
use ic_cdk_macros::*;
use crate::{registration::*, response::*, post::*};

#[query(name = "__get_candid_interface_tmp_hack")]
fn export_candid() -> String {
    export_service!();
    __export_service()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn save_candid() {
        use std::env;
        use std::fs::write;
        use std::path::PathBuf;

        let dir = PathBuf::from(env::current_dir().unwrap());
        write(dir.join("service.did"), export_candid()).expect("Write failed.");
    }
}