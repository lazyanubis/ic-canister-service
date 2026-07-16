//! https://github.com/dfinity/pocketic
use candid::encode_one;
use pocket_ic::{ErrorCode, PocketIcBuilder, RejectCode};

mod util;

mod service;

const WASM_MODULE_0_0_1: &[u8] = include_bytes!("../sources/source_opt_0_0_1.wasm.gz");
const WASM_MODULE_NEXT: &[u8] = include_bytes!("../sources/source_opt.wasm.gz");

#[ignore]
#[test]
#[rustfmt::skip]
fn test_upgrade() {
    // let pic = PocketIc::new();
    let pic = PocketIcBuilder::new().with_nns_subnet().build();

    let (default_identity, ..) = util::get_identity();

    let canister_id = pic.create_canister_with_settings(Some(default_identity), None);
    pic.add_cycles(canister_id, 20 * 10_u128.pow(12));
    // ! v0.0.1
    pic.install_canister(canister_id, WASM_MODULE_0_0_1.to_vec(), encode_one(Some(InitArgs::V1(InitArg { supers: None, schedule: None }))).unwrap(), Some(default_identity));

    use service::*;

    let pocketed_canister_id = PocketedCanisterId::new(canister_id, &pic);
    #[allow(unused)] let default = pocketed_canister_id.sender(default_identity);

    // ! next
    for _ in 0..6 { pic.tick(); } // 🕰︎
    let arg: Vec<u8> = encode_one(None::<()>).unwrap();
    assert_eq!(arg, vec![68, 73, 68, 76, 1, 110, 127, 1, 0, 0]); // 4449444c016e7f010000
    let reject = pic
        .upgrade_canister(canister_id, WASM_MODULE_NEXT.to_vec(), arg, Some(default_identity))
        .unwrap_err();
    assert_eq!(reject.reject_code, RejectCode::CanisterError);
    assert!(reject.reject_message.contains("Canister is running. Not paused."));
    assert_eq!(reject.error_code, ErrorCode::CanisterCalledTrap);
    assert!(reject.certified);

    // ! next
    for _ in 0..6 { pic.tick(); } // 🕰︎
    default.pause_replace(Some("test next".to_string())).unwrap();
    let arg: Vec<u8> = encode_one(None::<()>).unwrap();
    assert_eq!(arg, vec![68, 73, 68, 76, 1, 110, 127, 1, 0, 0]); // 4449444c016e7f010000
    pic.upgrade_canister(canister_id, WASM_MODULE_NEXT.to_vec(), arg, Some(default_identity)).unwrap();
    default.pause_replace(None).unwrap();

    // test_panic();
}

#[allow(unused)]
fn test_panic() {
    util::print_backtrace();
    panic!("test panic")
}
