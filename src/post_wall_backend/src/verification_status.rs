use std::cell::RefCell;

use candid::{Nat, CandidType, candid_method};
use ic_cdk_macros::*;
use ic_stable_memory::derive::{StableType, AsFixedSizeBytes};

use crate::{response::UpdateStateResponse, init::is_this_caller_authority};


#[derive(CandidType, StableType, AsFixedSizeBytes)]
pub struct VerificationCharge{
    pub regular_verification: Option<Nat>,
    pub plus_verification: Option<Nat>,
    pub business_verifcation: Option<Nat>,
}

impl Default for VerificationCharge{
    fn default() -> Self {
        Self{
            regular_verification: None,
            plus_verification: None,
            business_verifcation: None,
        }
    }
}

#[derive(CandidType, StableType, AsFixedSizeBytes, Default)]
pub struct UserCount{
    pub regular_verified_user: u128,
    pub plus_verified_user: u128,
    pub business_veried_user: u128,
}

impl UserCount{
    fn total_users(&self) -> u128{
        self.regular_verified_user + self.plus_verified_user + self.business_veried_user
    }
}

#[derive(CandidType, StableType, AsFixedSizeBytes, Clone)]
pub enum VerificationBadge{
    RegularVerified,
    PlusVerified,
    BusinessVerified,
}

thread_local! {
    pub static VERIFICATION_CHARGE: RefCell<VerificationCharge> = RefCell::default();
    pub static USER_COUNT: RefCell<UserCount> = RefCell::default();
}

pub(crate) fn _add_regular_user(){
    USER_COUNT.with(|state|{
        let state = &mut state.borrow_mut();
        state.regular_verified_user += 1;
    })
}

pub(crate) fn _remove_regular_user(){
    USER_COUNT.with(|state|{
        let state = &mut state.borrow_mut();
        state.regular_verified_user -= 1;
    })
}

pub(crate) fn _add_plus_verfied_user(){
    USER_COUNT.with(|state|{
        let state = &mut state.borrow_mut();
        state.plus_verified_user += 1;
    })
}

pub(crate) fn _remove_plus_verfied_user(){
    USER_COUNT.with(|state|{
        let state = &mut state.borrow_mut();
        state.plus_verified_user -= 1;
    })
}

pub(crate) fn _add_business_verified_user(){
    USER_COUNT.with(|state|{
        let state = &mut state.borrow_mut();
        state.business_veried_user += 1;
    })
}

pub(crate) fn _remove_business_verified_user(){
    USER_COUNT.with(|state|{
        let state = &mut state.borrow_mut();
        state.business_veried_user -= 1;
    })
}

pub(crate) fn _total_user_count() -> u128{
    USER_COUNT.with(|state| state.borrow().total_users())
}

pub(crate) fn _regular_verification_charge() -> Option<Nat>{
    VERIFICATION_CHARGE.with(|state| state.borrow().regular_verification.clone())
}

pub(crate) fn _plus_verification_charge() -> Option<Nat>{
    VERIFICATION_CHARGE.with(|state| state.borrow().plus_verification.clone())
}

pub(crate) fn _business_verification_charge() -> Option<Nat>{
    VERIFICATION_CHARGE.with(|state| state.borrow().business_verifcation.clone())
}

pub(crate) fn _update_plus_verification_charge(new_fee: Nat){
    VERIFICATION_CHARGE.with(|state| state.borrow_mut().plus_verification = Some(new_fee));
}

pub(crate) fn _update_business_verification_charge(new_fee: Nat){
    VERIFICATION_CHARGE.with(|state| state.borrow_mut().business_verifcation = Some(new_fee));
}

pub(crate) fn _query_regular_verified_user_count() -> u128{
    USER_COUNT.with(|state| state.borrow().regular_verified_user)
}

pub(crate) fn _query_plus_verified_user_count() -> u128{
    USER_COUNT.with(|state| state.borrow().plus_verified_user)
}

pub(crate) fn _query_business_verified_user_count() -> u128{
    USER_COUNT.with(|state| state.borrow().business_veried_user)
}

#[query]
#[candid_method(query)]
pub fn total_number_of_users() -> u128{
    _total_user_count()
}

#[update]
#[candid_method(update)]
pub fn update_plus_verification_charge(new_fee: Nat) -> UpdateStateResponse{
    let caller = ic_cdk::caller();
    if !is_this_caller_authority(&caller){
        UpdateStateResponse::Unauthorized
    }else{
        _update_plus_verification_charge(new_fee);
        UpdateStateResponse::Success
    }
}

#[update]
#[candid_method(update)]
pub fn update_business_verification_charge(new_fee: Nat) -> UpdateStateResponse{
    let caller = ic_cdk::caller();
    if !is_this_caller_authority(&caller){
        UpdateStateResponse::Unauthorized
    }else{
        _update_business_verification_charge(new_fee);
        UpdateStateResponse::Success
    }
}