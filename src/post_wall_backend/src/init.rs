use std::cell::RefCell;

use candid::{Principal, candid_method};
use ic_cdk_macros::*;
use ic_stable_memory::{derive::{AsFixedSizeBytes, StableType}, stable_memory_init};
use crate::response::UpdateStateResponse;

#[derive(AsFixedSizeBytes, StableType)]
pub struct InitData{
    pub authority: Principal,
}

impl Default for InitData{
    fn default() -> Self {
        Self { authority: Principal::from_slice(&[]) }
    }
}

thread_local! {
    pub static INIT_DATA: RefCell<InitData> = RefCell::default();
}

pub(crate) fn _query_authority() -> Principal{
    INIT_DATA.with(|state| state.borrow().authority.clone())
}

pub(crate) fn _change_authorty(new_authority: Principal){
    INIT_DATA.with(|state| state.borrow_mut().authority = new_authority);
}

pub(crate) fn is_this_caller_authority(caller: &Principal) -> bool{
    let authority = _query_authority();
    if *caller != authority{
        false
    }else{
        true
    }
}

#[init]
#[candid_method(init)]
pub fn init(){
    stable_memory_init();
    let caller = ic_cdk::caller();
    INIT_DATA.with(|state|{
        let state = &mut state.borrow_mut();
        state.authority = caller;
    })
}

#[update]
#[candid_method(update)]
pub fn change_authority(new_authority: Principal) -> UpdateStateResponse{
    let caller = ic_cdk::caller();
    let authority = _query_authority();
    if caller != authority{
        UpdateStateResponse::Unauthorized
    }else{
        _change_authorty(new_authority);
        UpdateStateResponse::Success
    }
}

#[query]
#[candid_method(query)]
pub fn query_authority() -> Principal{
    _query_authority()
}