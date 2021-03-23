//! Unit tests for the Stp258Standard module.

#![cfg(test)]

use super::*;
use frame_support::{assert_noop, assert_ok};
use mock::{Event, *};
use sp_runtime::traits::BadOrigin;

#[test]
fn stp258_currency_lockable_should_work() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob_n_serper_n_settpay()
		.build()
		.execute_with(|| {
			assert_ok!(Stp258Standard::set_lock(ID_1, SETT, &ALICE, 50 * 10_000));
			assert_eq!(Stp258Tokens::locks(&ALICE, SETT).len(), 1);
			assert_ok!(Stp258Standard::set_lock(ID_1, DNAR, &ALICE, 50));
			assert_eq!(PalletBalances::locks(&ALICE).len(), 1);
		});
}

#[test]
fn stp258_currency_reservable_should_work() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob_n_serper_n_settpay()
		.build()
		.execute_with(|| {
			assert_eq!(Stp258Standard::total_issuance(DNAR), 400);
			assert_eq!(Stp258Standard::total_issuance(SETT), 400 * 10_000);
			assert_eq!(Stp258Standard::free_balance(SETT, &ALICE), 100 * 10_000);
			assert_eq!(Stp258Native::free_balance(&ALICE), 100);

			assert_ok!(Stp258Standard::reserve(SETT, &ALICE, 30 * 10_000));
			assert_ok!(Stp258Standard::reserve(DNAR, &ALICE, 40));
			assert_eq!(Stp258Standard::reserved_balance(SETT, &ALICE), 30 * 10_000);
			assert_eq!(Stp258Standard::reserved_balance(DNAR, &ALICE), 40);
		});
}

#[test]
fn stp258_native_lockable_should_work() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob_n_serper_n_settpay()
		.build()
		.execute_with(|| {
			assert_ok!(Stp258Native::set_lock(ID_1, &ALICE, 10));
			assert_eq!(PalletBalances::locks(&ALICE).len(), 1);
			assert_ok!(Stp258Native::remove_lock(ID_1, &ALICE));
			assert_eq!(PalletBalances::locks(&ALICE).len(), 0);
		});
}

#[test]
fn stp258_native_reservable_should_work() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob_n_serper_n_settpay()
		.build()
		.execute_with(|| {
			assert_ok!(Stp258Native::reserve(&ALICE, 50));
			assert_eq!(Stp258Native::reserved_balance(&ALICE), 50);
		});
}

#[test]
fn stp258_asset_adapting_pallet_balances_lockable() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob_n_serper_n_settpay()
		.build()
		.execute_with(|| {
			assert_ok!(AdaptedStp258Asset::set_lock(ID_1, &ALICE, 10));
			assert_eq!(PalletBalances::locks(&ALICE).len(), 1);
			assert_ok!(AdaptedStp258Asset::remove_lock(ID_1, &ALICE));
			assert_eq!(PalletBalances::locks(&ALICE).len(), 0);
		});
}

#[test]
fn stp258_asset_adapting_pallet_balances_reservable() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob_n_serper_n_settpay()
		.build()
		.execute_with(|| {
			assert_ok!(AdaptedStp258Asset::reserve(&ALICE, 50));
			assert_eq!(AdaptedStp258Asset::reserved_balance(&ALICE), 50);
		});
}

#[test]
fn stp258_currency_should_work() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob_n_serper_n_settpay()
		.build()
		.execute_with(|| {
			assert_ok!(Stp258Standard::transfer(Some(ALICE).into(), BOB, SETT, 50 * 10_000));
			assert_eq!(Stp258Standard::free_balance(SETT, &ALICE), 50 * 10_000);
			assert_eq!(Stp258Standard::free_balance(SETT, &BOB), 150 * 10_000);
		});
}

#[test]
fn stp258_currency_extended_should_work() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob_n_serper_n_settpay()
		.build()
		.execute_with(|| {
			assert_ok!(<Stp258Standard as Stp258CurrencyExtended<AccountId>>::update_balance(
				SETT, &ALICE, 50 * 10_000
			));
			assert_eq!(Stp258Standard::free_balance(SETT, &ALICE), 150 * 10_000);
		});
}

#[test]
fn stp258_native_should_work() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob_n_serper_n_settpay()
		.build()
		.execute_with(|| {
			assert_ok!(Stp258Standard::transfer_native_currency(Some(ALICE).into(), BOB, 50));
			assert_eq!(Stp258Native::free_balance(&ALICE), 50);
			assert_eq!(Stp258Native::free_balance(&BOB), 150);

			assert_ok!(Stp258Native::transfer(&ALICE, &BOB, 10));
			assert_eq!(Stp258Native::free_balance(&ALICE), 40);
			assert_eq!(Stp258Native::free_balance(&BOB), 160);

			assert_eq!(Stp258Standard::slash(DNAR, &ALICE, 10), 0);
			assert_eq!(Stp258Native::free_balance(&ALICE), 30);
			assert_eq!(Stp258Native::total_issuance(), 390);
		});
}

#[test]
fn stp258_native_extended_should_work() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob_n_serper_n_settpay()
		.build()
		.execute_with(|| {
			assert_ok!(Stp258Native::update_balance(&ALICE, 10));
			assert_eq!(Stp258Native::free_balance(&ALICE), 110);

			assert_ok!(<Stp258Standard as Stp258CurrencyExtended<AccountId>>::update_balance(
				DNAR,
				&ALICE,
				10
			));
			assert_eq!(Stp258Native::free_balance(&ALICE), 120);
		});
}

#[test]
fn stp258_asset_adapting_pallet_balances_transfer() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob_n_serper_n_settpay()
		.build()
		.execute_with(|| {
			assert_ok!(AdaptedStp258Asset::transfer(&ALICE, &BOB, 50));
			assert_eq!(PalletBalances::total_balance(&ALICE), 50 );
			assert_eq!(PalletBalances::total_balance(&BOB), 150);
			assert_ok!(AdaptedStp258Asset::transfer(&ALICE, &SERPER, 10));
			assert_eq!(PalletBalances::total_balance(&ALICE), 40);
			assert_eq!(PalletBalances::total_balance(&SERPER), 110);
		});
}

#[test]
fn stp258_asset_adapting_pallet_balances_deposit() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob_n_serper_n_settpay()
		.build()
		.execute_with(|| {
			assert_ok!(AdaptedStp258Asset::deposit(&SERPER, 50));
			assert_eq!(PalletBalances::total_balance(&SERPER), 150);
			assert_eq!(PalletBalances::total_issuance(), 450);
		});
}

#[test]
fn stp258_asset_adapting_pallet_balances_withdraw() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob_n_serper_n_settpay()
		.build()
		.execute_with(|| {
			assert_ok!(AdaptedStp258Asset::withdraw(&ALICE, 100));
			assert_eq!(PalletBalances::total_balance(&ALICE), 0);
			assert_eq!(PalletBalances::total_issuance(), 300);
		});
}

#[test]
fn stp258_asset_adapting_pallet_balances_slash() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob_n_serper_n_settpay()
		.build()
		.execute_with(|| {
			assert_eq!(AdaptedStp258Asset::slash(&ALICE, 101), 1);
			assert_eq!(PalletBalances::total_balance(&ALICE), 0);
			assert_eq!(PalletBalances::total_issuance(), 300);
		});
}

#[test]
fn stp258_asset_adapting_pallet_balances_update_balance() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob_n_serper_n_settpay()
		.build()
		.execute_with(|| {
			assert_ok!(AdaptedStp258Asset::update_balance(&ALICE, -10));
			assert_eq!(PalletBalances::total_balance(&ALICE), 90);
			assert_eq!(PalletBalances::total_issuance(), 390);
		});
}

#[test]
fn update_balance_call_should_work() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob_n_serper_n_settpay()
		.build()
		.execute_with(|| {
			assert_ok!(Stp258Standard::update_balance(
				Origin::root(),
				ALICE,
				DNAR,
				-10
			));
			assert_eq!(Stp258Native::free_balance(&ALICE), 90);
			assert_eq!(Stp258Standard::free_balance(SETT, &ALICE), 100 * 10_000);
			assert_ok!(Stp258Standard::update_balance(Origin::root(), ALICE, SETT, 10 * 10_000));
			assert_eq!(Stp258Standard::free_balance(SETT, &ALICE), 110 * 10_000);
		});
}

#[test]
fn update_balance_call_fails_if_not_root_origin() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			Stp258Standard::update_balance(Some(ALICE).into(), ALICE, SETT, 100 * 10_000),
			BadOrigin
		);
	});
}

#[test]
fn call_event_should_work() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob_n_serper_n_settpay()
		.build()
		.execute_with(|| {
			System::set_block_number(1);

			assert_ok!(Stp258Standard::transfer(Some(ALICE).into(), BOB, SETT, 50 * 10_000));
			assert_eq!(Stp258Standard::free_balance(SETT, &ALICE), 50 * 10_000);
			assert_eq!(Stp258Standard::free_balance(SETT, &BOB), 150 * 10_000);

			let transferred_event = Event::stp258_standard(crate::Event::Transferred(SETT, ALICE, BOB, 50 * 10_000));
			assert!(System::events().iter().any(|record| record.event == transferred_event));

			assert_ok!(<Stp258Standard as Stp258Currency<AccountId>>::transfer(
				SETT, &ALICE, &BOB, 10 * 10_000
			));
			assert_eq!(Stp258Standard::free_balance(SETT, &ALICE), 40 * 10_000);
			assert_eq!(Stp258Standard::free_balance(SETT, &BOB), 160 * 10_000);

			let transferred_event = Event::stp258_standard(crate::Event::Transferred(SETT, ALICE, BOB, 10 * 10_000));
			assert!(System::events().iter().any(|record| record.event == transferred_event));

			assert_ok!(<Stp258Standard as Stp258Currency<AccountId>>::deposit(
				SETT, &ALICE, 100 * 10_000
			));
			assert_eq!(Stp258Standard::free_balance(SETT, &ALICE), 140 * 10_000);

			let transferred_event = Event::stp258_standard(crate::Event::Deposited(SETT, ALICE, 100 * 10_000));
			assert!(System::events().iter().any(|record| record.event == transferred_event));

			assert_ok!(<Stp258Standard as Stp258Currency<AccountId>>::withdraw(
				SETT, &ALICE, 20 * 10_000
			));
			assert_eq!(Stp258Standard::free_balance(SETT, &ALICE), 120 * 10_000);

			let transferred_event = Event::stp258_standard(crate::Event::Withdrawn(SETT, ALICE, 20 * 10_000));
			assert!(System::events().iter().any(|record| record.event == transferred_event));
		});
}
