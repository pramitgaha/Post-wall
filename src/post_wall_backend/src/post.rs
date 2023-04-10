use std::cell::RefCell;

use candid::{CandidType, candid_method, Deserialize};
use ic_stable_memory::{SBox, derive::{AsFixedSizeBytes, StableType}, collections::{SHashMap, SVec}};
use crate::{response::CreatePostResponse, registration::get_username};
use ic_cdk_macros::*;

#[derive(CandidType, AsFixedSizeBytes, StableType, Default, Clone, Copy)]
pub struct ReactionCount{
    pub like: u128,
    pub heart: u128,
    pub dislike: u128,
}

#[derive(CandidType, AsFixedSizeBytes, StableType)]
pub struct Post{
    pub topic: SBox<String>,
    pub message: SBox<String>,
    pub posted_at: u64,
    pub reaction_count: ReactionCount,
}

impl Post{
    pub const MAX_TOPIC_LENGTH: usize = 50;
    pub const MAX_MESSAGE_LENGTH: usize = 700;

    fn _post(create_post_data: CreatePostData) -> Option<Self>{
        let topic = match SBox::new(create_post_data.topic){
            Ok(topic) => topic,
            Err(_) => return None
        };
        let message = match SBox::new(create_post_data.message){
            Ok(message) => message,
            Err(_) => return None
        };
        let post = Self{
            topic,
            message,
            posted_at: ic_cdk::api::time(),
            reaction_count: ReactionCount::default(),
        };
        Some(post)
    }

    fn clone(&self) -> Option<Self>{
        let topic = match SBox::new(self.topic.clone()){
            Ok(topic) => topic,
            Err(_) => return None
        };
        let message = match SBox::new(self.message.clone()){
            Ok(message) => message,
            Err(_) => return None,
        };
        let cloned_data = Self{
            topic,
            message,
            posted_at: self.posted_at,
            reaction_count: self.reaction_count
        };
        Some(cloned_data)
    }
}

#[derive(StableType, AsFixedSizeBytes)]
pub struct PostState{
    pub posts: SHashMap<SBox<String>, SVec<Post>>
}

impl Default for PostState{
    fn default() -> Self {
        Self{
            posts: SHashMap::new(),
        }
    }
}

thread_local! {
    pub static POST_STATE: RefCell<PostState> = RefCell::default()
}

pub(crate) fn _allocate_storage(user: SBox<String>){
    POST_STATE.with(|state|{
        let state = &mut state.borrow_mut();
        let _ = state.posts.insert(user, SVec::new());
    })
}

pub(crate) fn _insert_post(user: SBox<String>, new_post: Post) -> Result<(), CreatePostResponse>{
    POST_STATE.with(|state|{
        let state = &mut state.borrow_mut();
        let mut current_posts = match state.posts.get(&user){
            None => return Err(CreatePostResponse::UserNotRegistered),
            Some(old_posts) => {
                let mut s_vec = match SVec::new_with_capacity(state.posts.len()){
                    Ok(list) => list,
                    Err(_) => return Err(CreatePostResponse::FailedToAllocateMemory),
                };
                for p in old_posts.iter(){
                    let p = match p.clone(){
                        None => return Err(CreatePostResponse::FailedToAllocateMemory),
                        Some(p) => p
                    };
                    match s_vec.push(p){
                        Ok(_) => continue,
                        Err(_) => return Err(CreatePostResponse::FailedToAllocateMemory)
                    }
                }
                s_vec
            }
        };
        if current_posts.push(new_post).is_err(){
            return Err(CreatePostResponse::FailedToAllocateMemory)
        }
        match state.posts.insert(user, current_posts){
            Ok(_) => Ok(()),
            Err(_) => Err(CreatePostResponse::FailedToAllocateMemory)
        }
    })
}

#[derive(CandidType, Deserialize)]
pub struct CreatePostData{
    pub topic: String,
    pub message: String,
}

fn create_post_check(create_post_data: &CreatePostData) -> Result<(), CreatePostResponse>{
    if create_post_data.topic.len() > Post::MAX_TOPIC_LENGTH{
        Err(CreatePostResponse::TopicTooLong)
    }else if create_post_data.message.len() > Post::MAX_MESSAGE_LENGTH{
        Err(CreatePostResponse::MessageTooLong)
    }else{
        Ok(())
    }
}

#[update]
#[candid_method(update)]
pub fn create_post(create_post_data: CreatePostData) -> CreatePostResponse{
    let caller = ic_cdk::caller();
    let username = match get_username(&caller){
        None => return CreatePostResponse::UserNotRegistered,
        Some(username) => username
    };
    let username = match SBox::new(username){
        Ok(name) => name,
        Err(_) => return CreatePostResponse::FailedToAllocateMemory
    };
    if let Err(e) = create_post_check(&create_post_data){
        return e
    }
    let post = match Post::_post(create_post_data){
        None => return CreatePostResponse::FailedToAllocateMemory,
        Some(post) => post
    };
    if let Err(e) = _insert_post(username, post){
        return e
    }
    CreatePostResponse::Success
}