//! Unit tests for the stp258 module.

#![cfg(test)]
use super::*;
use frame_support::{assert_noop, assert_ok};
use mock::*;
use sp_core::H160;
use sp_runtime::traits::BadOrigin;

use traits::SettCurrency;

#[test]
fn it_works_for_default_value() {
    new_test_ext().execute_with(|| {
        // Dispatch a signed extrinsic.
        assert_ok!(Stp258::do_something(Origin::signed(1), 42));
        // Read pallet storage and assert an expected result.
        assert_eq!(Stp258::something(), Some(42));
    });
}

#[test]
fn correct_error_for_none_value() {
    new_test_ext().execute_with(|| {
        // Ensure the expected error is thrown when no value is present.
        assert_noop!(
            Stp258::cause_error(Origin::signed(1)),
            Error::<Test>::NoneValue
        );
    });
}

// ------------------------------------------------------------


#[test]
fn sett_currency_should_work() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob()
		.build()
		.execute_with(|| {
			assert_ok!(Stp258::transfer(Some(ALICE).into(), BOB, SETT_USD_ID, 50));
			assert_eq!(Stp258::free_balance(SETT_USD_ID, &ALICE), 50);
			assert_eq!(Stp258::free_balance(SETT_USD_ID, &BOB), 150);
		});
}

#[test]
fn sett_currency_extended_should_work() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob()
		.build()
		.execute_with(|| {
			assert_ok!(<Stp258 as ExtendedSettCurrency<AccountId>>::update_balance(
				SETT_USD_ID, &ALICE, 50
			));
			assert_eq!(Stp258::free_balance(SETT_USD_ID, &ALICE), 150);
		});
}

#[test]
fn sett_currency_minting_should_work() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob()
		.build()
		.execute_with(|| { 
			assert_ok!(Stp258::mint(SETT_USD_ID, &ALICE, 10));
			assert_eq!(Stp258::free_balance(SETT_USD_ID, &ALICE), 110);
			assert_ok!(Stp258::mint(SETT_USD_ID, &BOB, 5));
			assert_eq!(Stp258::free_balance(SETT_USD_ID, &BOB), 105);
		});
}

#[test]

fn sett_currency_burning_should_work() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob()
		.build()
		.execute_with(|| {
			assert_ok!(Stp258::burn(SETT_USD_ID, &ALICE, 10));
			assert_eq!(Stp258::free_balance(SETT_USD_ID, &ALICE), 90);
			assert_ok!(Stp258::burn(SETT_USD_ID, &BOB, 5));
			assert_eq!(Stp258::free_balance(SETT_USD_ID, &BOB), 95);
		});
}

#[test]
fn native_currency_should_work() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob()
		.build()
		.execute_with(|| {
			assert_ok!(Stp258::transfer_native_currency(Some(ALICE).into(), BOB, 50));
			assert_eq!(NativeCurrency::free_balance(&ALICE), 50);
			assert_eq!(NativeCurrency::free_balance(&BOB), 150);

			assert_ok!(NativeCurrency::transfer(&ALICE, &BOB, 10));
			assert_eq!(NativeCurrency::free_balance(&ALICE), 40);
			assert_eq!(NativeCurrency::free_balance(&BOB), 160);

			assert_eq!(Stp258::slash(NATIVE_SETT_USD_ID, &ALICE, 10), 0);
			assert_eq!(NativeCurrency::free_balance(&ALICE), 30);
			assert_eq!(NativeCurrency::total_issuance(), 190);
		});
}

#[test]
fn native_currency_extended_should_work() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob()
		.build()
		.execute_with(|| {
			assert_ok!(NativeCurrency::update_balance(&ALICE, 10));
			assert_eq!(NativeCurrency::free_balance(&ALICE), 110);

			assert_ok!(<Stp258 as ExtendedSettCurrency<AccountId>>::update_balance(
				NATIVE_SETT_USD_ID,
				&ALICE,
				10
			));
			assert_eq!(NativeCurrency::free_balance(&ALICE), 120);
		});
}

#[test]
fn native_currency_minting_should_work() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob()
		.build()
		.execute_with(|| { 
			assert_ok!(Stp258::mint(DNAR, &ALICE, 10));
			assert_eq!(Stp258::free_balance(DNAR, &ALICE), 110);
			assert_ok!(Stp258::mint(DNAR, &BOB, 5));
			assert_eq!(Stp258::free_balance(DNAR, &BOB), 105);
		});
}

#[test]

fn native_currency_burning_should_work() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob()
		.build()
		.execute_with(|| {
			assert_ok!(Stp258::burn(DNAR, &ALICE, 10));
			assert_eq!(Stp258::free_balance(DNAR, &ALICE), 90);
			assert_ok!(Stp258::burn(DNAR, &BOB, 5));
			assert_eq!(Stp258::free_balance(DNAR, &BOB), 95);
		});
}

#[test]
fn basic_currency_adapting_pallet_balances_transfer() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob()
		.build()
		.execute_with(|| {
			assert_ok!(AdaptedBasicCurrency::transfer(&ALICE, &BOB, 50));
			assert_eq!(PalletBalances::total_balance(&ALICE), 50);
			assert_eq!(PalletBalances::total_balance(&BOB), 150);

			// creation fee
			assert_ok!(AdaptedBasicCurrency::transfer(&ALICE, &EVA, 10));
			assert_eq!(PalletBalances::total_balance(&ALICE), 40);
			assert_eq!(PalletBalances::total_balance(&EVA), 10);
		});
}

#[test]
fn basic_currency_adapting_pallet_balances_deposit() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob()
		.build()
		.execute_with(|| {
			assert_ok!(AdaptedBasicCurrency::deposit(&EVA, 50));
			assert_eq!(PalletBalances::total_balance(&EVA), 50);
			assert_eq!(PalletBalances::total_issuance(), 250);
		});
}

#[test]
fn basic_currency_adapting_pallet_balances_withdraw() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob()
		.build()
		.execute_with(|| {
			assert_ok!(AdaptedBasicCurrency::withdraw(&ALICE, 100));
			assert_eq!(PalletBalances::total_balance(&ALICE), 0);
			assert_eq!(PalletBalances::total_issuance(), 100);
		});
}

#[test]
fn basic_currency_adapting_pallet_balances_slash() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob()
		.build()
		.execute_with(|| {
			assert_eq!(AdaptedBasicCurrency::slash(&ALICE, 101), 1);
			assert_eq!(PalletBalances::total_balance(&ALICE), 0);
			assert_eq!(PalletBalances::total_issuance(), 100);
		});
}

#[test]
fn basic_currency_adapting_pallet_balances_update_balance() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob()
		.build()
		.execute_with(|| {
			assert_ok!(AdaptedBasicCurrency::update_balance(&ALICE, -10));
			assert_eq!(PalletBalances::total_balance(&ALICE), 90);
			assert_eq!(PalletBalances::total_issuance(), 190);
		});
}

#[test]
fn lockable_sett_currency_should_work() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob()
		.build()
		.execute_with(|| {
			assert_ok!(Stp258::set_lock(ID_1, SETT_USD_ID, &ALICE, 50));
			assert_eq!(Tokens::locks(&ALICE, SETT_USD_ID).len(), 1);
			assert_ok!(Stp258::set_lock(ID_1, SETT_USD_ID, &ALICE, 50));
			assert_eq!(PalletBalances::locks(&ALICE).len(), 1);
		});
}

#[test]
fn reservable_sett_currency_should_work() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob()
		.build()
		.execute_with(|| {
			assert_eq!(Stp258::total_issuance(SETT_USD_ID), 200);
			assert_eq!(Stp258::total_issuance(SETT_USD_ID), 200);
			assert_eq!(Stp258::free_balance(SETT_USD_ID, &ALICE), 100);
			assert_eq!(NativeCurrency::free_balance(&ALICE), 100);

			assert_ok!(Stp258::reserve(SETT_USD_ID, &ALICE, 30));
			assert_ok!(Stp258::reserve(SETT_USD_ID, &ALICE, 40));
			assert_eq!(Stp258::reserved_balance(SETT_USD_ID, &ALICE), 30);
			assert_eq!(Stp258::reserved_balance(SETT_USD_ID, &ALICE), 40);
		});
}

#[test]
fn settswap_in_basic_currency_should_work() {
	// A generates a random proof. Keep it secret.
	let proof: [u8; 2] = [4, 2];
	// The hashed proof is the blake2_256 hash of the proof. This is public.
	let hashed_proof = blake2_256(&proof);
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob()
		.build()
		.execute_with(|| {
			// Alice creates the swap.
			assert_eq!(SettSwap::create_swap(Origin::signed(&ALICE), &BOB, hashed_proof.clone(), SettSwap::new(50), 1000));

			assert_eq!(PalletBalances::free_balance(&ALICE), 50);
			assert_eq!(PalletBalances::free_balance(&BOB), 200);

			// Bob uses the revealed proof to claim the swap.
			assert_eq!(SettSwap::claim_swap( Origin::signed(&BOB), proof.to_vec(), SettSwap::new(50)));

			assert_eq!(PalletBalances::free_balance(&ALICE), 50);
			assert_eq!(PalletBalances::free_balance(&BOB), 250);
		});
}

#[test]
fn settswap_in_native_currency_should_work() {
	// A generates a random proof. Keep it secret.
	let proof: [u8; 2] = [4, 2];
	// The hashed proof is the blake2_256 hash of the proof. This is public.
	let hashed_proof = blake2_256(&proof);
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob()
		.build()
		.execute_with(|| {
			// Bob creates the swap 2.
			assert_eq!(SettSwap::create_swap(Origin::signed(&BOB), &ALICE, hashed_proof.clone(), SettSwap::new(75), 1000));

			assert_eq!(NativeCurrency::free_balance(&ALICE), 100);
			assert_eq!(NativeCurrency::free_balance(&BOB), 125);

			// Alice reveals the proof and claims the swap 2.
			assert_eq!(SettSwap::claim_swap( Origin::signed(&ALICE), proof.to_vec(), SettSwap::new(75)));

			assert_eq!(NativeCurrency::free_balance(&ALICE), 175);
			assert_eq!(NativeCurrency::free_balance(&BOB), 125);

		});
}

#[test]
fn settswap_in_sett_currency_should_work() {
	// A generates a random proof. Keep it secret.
	let proof: [u8; 2] = [4, 2];
	// The hashed proof is the blake2_256 hash of the proof. This is public.
	let hashed_proof = blake2_256(&proof);
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob()
		.build()
		.execute_with(|| {
			// Bob creates the swap 2.
			assert_eq!(SettSwap::create_swap(Origin::signed(&BOB), &ALICE, hashed_proof.clone(), SettSwap::new(75), 1000));

			assert_eq!(Stp258::free_balance(&ALICE), 100);
			assert_eq!(Stp258::free_balance(&BOB), 125);

			// Alice reveals the proof and claims the swap 2.
			assert_eq!(SettSwap::claim_swap( Origin::signed(&ALICE), proof.to_vec(), SettSwap::new(75)));

			assert_eq!(Stp258::free_balance(&ALICE), 175);
			assert_eq!(Stp258::free_balance(&BOB), 125);

		});
}

#[test]
fn native_currency_lockable_should_work() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob()
		.build()
		.execute_with(|| {
			assert_ok!(NativeCurrency::set_lock(ID_1, &ALICE, 10));
			assert_eq!(PalletBalances::locks(&ALICE).len(), 1);
			assert_ok!(NativeCurrency::remove_lock(ID_1, &ALICE));
			assert_eq!(PalletBalances::locks(&ALICE).len(), 0);
		});
}

#[test]
fn native_currency_reservable_should_work() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob()
		.build()
		.execute_with(|| {
			assert_ok!(NativeCurrency::reserve(&ALICE, 50));
			assert_eq!(NativeCurrency::reserved_balance(&ALICE), 50);
		});
}

#[test]
fn basic_currency_adapting_pallet_balances_lockable() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob()
		.build()
		.execute_with(|| {
			assert_ok!(AdaptedBasicCurrency::set_lock(DNAR, &ALICE, 10));
			assert_eq!(PalletBalances::locks(&ALICE).len(), 1);
			assert_ok!(AdaptedBasicCurrency::remove_lock(DNAR, &ALICE));
			assert_eq!(PalletBalances::locks(&ALICE).len(), 0);
		});
}

#[test]
fn basic_currency_adapting_pallet_balances_reservable() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob()
		.build()
		.execute_with(|| {
			assert_ok!(AdaptedBasicCurrency::reserve(&ALICE, 50));
			assert_eq!(AdaptedBasicCurrency::reserved_balance(&ALICE), 50);
		});
}

#[test]
fn update_balance_call_should_work() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob()
		.build()
		.execute_with(|| {
			assert_ok!(Stp258::update_balance(
				Origin::root(),
				ALICE,
				NATIVE_SETT_USD_ID,
				-10
			));
			assert_eq!(NativeCurrency::free_balance(&ALICE), 90);
			assert_eq!(Stp258::free_balance(SETT_USD_ID, &ALICE), 100);
			assert_ok!(Stp258::update_balance(Origin::root(), ALICE, SETT_USD_ID, 10));
			assert_eq!(Stp258::free_balance(SETT_USD_ID, &ALICE), 110);
		});
}

#[test]
fn update_balance_call_fails_if_not_root_origin() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			Stp258::update_balance(Some(ALICE).into(), ALICE, SETT_USD_ID, 100),
			BadOrigin
		);
	});
}
DNAR
#[test]
fn call_event_should_work() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob()
		.build()
		.execute_with(|| {
			System::set_block_number(1);

			assert_ok!(Stp258::transfer(Some(ALICE).into(), BOB, SETT_USD_ID, 50));
			assert_eq!(Stp258::free_balance(SETT_USD_ID, &ALICE), 50);
			assert_eq!(Stp258::free_balance(SETT_USD_ID, &BOB), 150);

			let transferred_event = TestEvent::stp258(RawEvent::Transferred(SETT_USD_ID, ALICE, BOB, 50));
			assert!(System::events().iter().any(|record| record.event == transferred_event));

			assert_ok!(<Stp258 as SettCurrency<AccountId>>::transfer(
				SETT_USD_ID, &ALICE, &BOB, 10
			));
			assert_eq!(Stp258::free_balance(SETT_USD_ID, &ALICE), 40);
			assert_eq!(Stp258::free_balance(SETT_USD_ID, &BOB), 160);

			let transferred_event = TestEvent::stp258(RawEvent::Transferred(SETT_USD_ID, ALICE, BOB, 10));
			assert!(System::events().iter().any(|record| record.event == transferred_event));

			assert_ok!(<Stp258 as SettCurrency<AccountId>>::deposit(
				SETT_USD_ID, &ALICE, 100
			));
			assert_eq!(Stp258::free_balance(SETT_USD_ID, &ALICE), 140);

			let transferred_event = TestEvent::stp258(RawEvent::Deposited(SETT_USD_ID, ALICE, 100));
			assert!(System::events().iter().any(|record| record.event == transferred_event));

			assert_ok!(<Stp258 as SettCurrency<AccountId>>::withdraw(
				SETT_USD_ID, &ALICE, 20
			));
			assert_eq!(Stp258::free_balance(SETT_USD_ID, &ALICE), 120);

			let transferred_event = TestEvent::stp258(RawEvent::Withdrawn(SETT_USD_ID, ALICE, 20));
			assert!(System::events().iter().any(|record| record.event == transferred_event));
		});
}



