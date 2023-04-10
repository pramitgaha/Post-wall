use std::cell::RefCell;

use candid::{CandidType, candid_method, Deserialize, Principal};
use ic_cdk_macros::*;
use ic_stable_memory::{derive::{AsFixedSizeBytes, StableType}, SBox, collections::SVec};

use crate::{verification_status::VerificationBadge, response::RegistrationResponse,};

#[derive(CandidType, AsFixedSizeBytes, StableType, Clone, Deserialize)]
pub enum Gender{
    Male,
    Female,
    RatherNotToSay,
}

#[derive(StableType, AsFixedSizeBytes)]
pub struct Profile{
    pub first_name: SBox<String>,
    pub middle_name: SBox<String>,
    pub last_name: SBox<String>,
    pub username: SBox<String>,
    pub address: Principal,
    pub date_of_birth: u64,
    pub gender: Gender,
    pub verification_badge: VerificationBadge,
    pub number_of_posts: u128,
}

#[derive(CandidType)]
pub struct ProfileQuery{
    pub first_name: String,
    pub middle_name: String,
    pub last_name: String,
    pub username: String,
    pub date_of_birth: u64,
    pub gender: Gender,
    pub verification_badge: VerificationBadge,
    pub number_of_posts: u128,
}

impl Profile{
    pub const MAXIMUM_FIRST_NAME_LENGTH: usize = 15;
    pub const MAXIMUM_MIDDLE_NAME_LENGTH: usize = 10;
    pub const MAXIMUM_LAST_NAME_LENGTH: usize = 15;
    pub const MAXIMUM_USERNAME_LENGTH: usize = 20;

    fn new_profile((registration_data, verification_badge, address): (RegistrationData, VerificationBadge, Principal)) -> Option<Profile>{
        let first_name = match SBox::new(registration_data.first_name){
            Ok(first_name) => first_name,
            Err(_) => return None
        };
        let middle_name = match SBox::new(registration_data.middle_name){
            Ok(middle_name) => middle_name,
            Err(_) => return None,
        };
        let last_name = match SBox::new(registration_data.last_name){
            Ok(last_name) => last_name,
            Err(_) => return None
        };
        let username = match SBox::new(registration_data.username){
            Ok(username) => username,
            Err(_) => return None
        };
        let profile_data = Profile{
            first_name,
            middle_name,
            last_name,
            username,
            address,
            date_of_birth: registration_data.date_of_birth,
            gender: registration_data.gender,
            verification_badge,
            number_of_posts: 0,
        };
        Some(profile_data)
    }

    fn _to_profile_query(&self) -> ProfileQuery{
        ProfileQuery{
            first_name: self.first_name.clone(),
            middle_name: self.middle_name.clone(),
            last_name: self.last_name.clone(),
            username: self.username.clone(),
            date_of_birth: self.date_of_birth,
            gender: self.gender.clone(),
            verification_badge: self.verification_badge.clone(),
            number_of_posts: self.number_of_posts,
        }
    }

    fn _update_first_name(&mut self, new_first_name: SBox<String>){
        self.first_name = new_first_name;
    }

    fn _update_middle_name(&mut self, new_middle_name: SBox<String>){
        self.middle_name = new_middle_name;
    }

    fn _update_last_name(&mut self, new_last_name: SBox<String>){
        self.last_name = new_last_name;
    }

    fn _increment_post_count(&mut self){
        self.number_of_posts += 1;
    }

    fn _decrement_post_count(&mut self){
        self.number_of_posts -= 1;
    }

    fn _change_verification_badge(&mut self, new_verificiation_badge: VerificationBadge){
        self.verification_badge = new_verificiation_badge;
    }
}

pub struct UserProfiles{
    pub users: SVec<Profile>
}

impl Default for UserProfiles{
    fn default() -> Self {
        Self { users: SVec::new() }
    }
}

impl UserProfiles{
    fn _user_name_taken(&self, user_name: SBox<String>) -> bool{
        if self.users.binary_search_by(|user| user.username.cmp(&user_name)).is_ok(){
            true
        }else{
            false
        }
    }

    fn is_this_address_already_registered(&self, address: &Principal) -> bool{
        if self.users.binary_search_by(|user| user.address.cmp(address)).is_ok(){
            true
        }else{
            false
        }
    }

    fn get_username(&self, address: &Principal) -> Option<String>{
        let index = match self.users.binary_search_by(|user| user.address.cmp(address)){
            Ok(index) => index,
            Err(_) => return None
        };
        match self.users.get(index){
            None => return None,
            Some(user) => Some(user.username.clone())
        }
    }
}

thread_local! {
    pub static USER_PROFILE: RefCell<UserProfiles> = RefCell::default();
}

fn _register_profile(profile: Profile) -> bool{
    USER_PROFILE.with(|state|{
        let state = &mut state.borrow_mut();
        match state.users.push(profile){
            Ok(_) => true,
            Err(_) => false
        }
    })
}

fn _username_check(username: SBox<String>) -> bool{
    USER_PROFILE.with(|state| state.borrow()._user_name_taken(username))
}

fn is_this_user_already_registered(address: &Principal) -> bool{
    USER_PROFILE.with(|state| state.borrow().is_this_address_already_registered(address))
}

pub(crate) fn get_username(address: &Principal) -> Option<String>{
    USER_PROFILE.with(|state| state.borrow().get_username(address))
}

#[derive(CandidType, Deserialize)]
pub struct RegistrationData{
    pub first_name: String,
    pub middle_name: String,
    pub last_name: String,
    pub username: String,
    pub date_of_birth: u64,
    pub gender: Gender,
}

#[update]
#[candid_method(update)]
pub fn register_as_regular_user(registration_data: RegistrationData) -> RegistrationResponse{
    let caller = ic_cdk::caller();
    if is_this_user_already_registered(&caller){
        return RegistrationResponse::AlreadyRegistered
    }
    if let Err(e) = registration_check(&registration_data){
        return e
    }
    let username = registration_data.username.clone();
    let boxed_username = match SBox::new(username.clone()){
        Ok(username) => username,
        Err(_) => return RegistrationResponse::FailedToAllocateMemory
    };
    if _username_check(boxed_username){
        return RegistrationResponse::UserNameTaken
    }
    let profile = match Profile::new_profile((registration_data, VerificationBadge::RegularVerified, caller)){
        Some(profile) => profile,
        None => return RegistrationResponse::FailedToAllocateMemory
    };
    _register_profile(profile);
    RegistrationResponse::Success { username }
}

#[update]
#[candid_method(update)]
pub fn register_as_plus_user(registration_data: RegistrationData) -> RegistrationResponse{
    let caller = ic_cdk::caller();
    if is_this_user_already_registered(&caller){
        return RegistrationResponse::AlreadyRegistered
    }
    if let Err(e) = registration_check(&registration_data){
        return e
    }
    let username = registration_data.username.clone();
    let boxed_username = match SBox::new(username.clone()){
        Ok(username) => username,
        Err(_) => return RegistrationResponse::FailedToAllocateMemory
    };
    if _username_check(boxed_username){
        return RegistrationResponse::UserNameTaken
    }
    let profile = match Profile::new_profile((registration_data, VerificationBadge::PlusVerified, caller)){
        Some(profile) => profile,
        None => return RegistrationResponse::FailedToAllocateMemory
    };
    _register_profile(profile);
    RegistrationResponse::Success { username }
}

#[update]
#[candid_method(update)]
pub fn register_as_business_account(registration_data: RegistrationData) -> RegistrationResponse{
    let caller = ic_cdk::caller();
    if is_this_user_already_registered(&caller){
        return RegistrationResponse::AlreadyRegistered
    }
    if let Err(e) = registration_check(&registration_data){
        return e
    }
    let username = registration_data.username.clone();
    let boxed_username = match SBox::new(username.clone()){
        Ok(username) => username,
        Err(_) => return RegistrationResponse::FailedToAllocateMemory
    };
    if _username_check(boxed_username){
        return RegistrationResponse::UserNameTaken
    }
    let profile = match Profile::new_profile((registration_data, VerificationBadge::BusinessVerified, caller)){
        Some(profile) => profile,
        None => return RegistrationResponse::FailedToAllocateMemory
    };
    _register_profile(profile);
    RegistrationResponse::Success { username }
}

fn registration_check(registration_data: &RegistrationData) -> Result<(), RegistrationResponse>{
    if registration_data.first_name.len() > Profile::MAXIMUM_FIRST_NAME_LENGTH{
        Err(RegistrationResponse::FirstNameTooLong)
    }else if registration_data.middle_name.len() > Profile::MAXIMUM_MIDDLE_NAME_LENGTH{
        Err(RegistrationResponse::MiddleNameTooLong)
    }else if registration_data.last_name.len() > Profile::MAXIMUM_LAST_NAME_LENGTH{
        Err(RegistrationResponse::LastNameTooLong)
    }else{
        Ok(())
    }
}