use ic_ledger_types::Subaccount;
use candid::Principal;

pub(crate) fn subaccount_generator(principal: &Principal) -> Subaccount{
    let mut subaccount = [0; 32];
    let slice = principal.as_slice();
    subaccount[0] = slice.len() as u8;
    subaccount[1..slice.len() + 1].copy_from_slice(slice);
    Subaccount(subaccount)
}