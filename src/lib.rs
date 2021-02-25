//! # stp258 Module
//!
//! ## Overview
//!
//! The stp258 module provides a mixed stablecoin system, by configuring a
//! native currency which implements `BasicCurrencyExtended`, and a
//! multi-currency which implements `SettCurrency`.
//!
//! It also provides an adapter, to adapt `frame_support::traits::Currency`
//! implementations into `BasicCurrencyExtended`.
//!
//! The stp258 module provides functionality of both `ExtendedSettCurrency`
//! and `BasicCurrencyExtended`, via unified interfaces, and all calls would be
//! delegated to the underlying multi-currency and base currency system.
//! A native currency ID could be set by `Config::GetNativeCurrencyId`, to
//! identify the native currency.
//!
//! ### Implementations
//!
//! The stp258 module provides implementations for following traits.
//!
//! - `SettCurrency` - Abstraction over a fungible multi-currency stablecoin system 
//! that includes `basket_token`.
//! - `ExtendedSettCurrency` - Extended `SettCurrency` with additional helper
//!   types and methods, like updating balance
//! by a given signed integer amount.
//!
//! ## Interface
//!
//! ### Dispatchable Functions
//!
//! - `transfer` - Transfer some balance to another account, in a given
//!   currency.
//! - `transfer_native_currency` - Transfer some balance to another account, in
//!   native currency set in
//! `Config::NativeCurrency`.
//! - `update_balance` - Update balance by signed integer amount, in a given
//!   currency, root origin required.
//!
//! - `mint` - Mint some amount to some given account, in a given
//!   currency.

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]

use codec::Codec;
use frame_support::{
	pallet_prelude::*,
	traits::{
		Currency as PalletCurrency, 
		ExistenceRequirement, Get, 
		LockableCurrency as PalletLockableCurrency,
		ReservableCurrency as PalletReservableCurrency, 
		WithdrawReasons,
	},
	weights::Weight,
};
use frame_system::{ensure_root, ensure_signed, pallet_prelude::*};
use orml_traits::{
	account::MergeAccount,
	arithmetic::{Signed, SimpleArithmetic},
	BalanceStatus, 
	BasicCurrency, 
	BasicCurrencyExtended, 
	BasicLockableCurrency, 
	BasicReservableCurrency,
	LockIdentifier, 
	MultiCurrency as SettCurrency, 
	MultiCurrencyExtended as ExtendedSettCurrency, 
	MultiLockableCurrency as LockableSettCurrency, 
	MultiReservableCurrency as ReservableSettCurrency,
};
use orml_utilities::with_transaction_result;
use sp_runtime::{
	traits::{CheckedSub, MaybeSerializeDeserialize, StaticLookup, Zero},
	DispatchError, 
	DispatchResult,
};
use sp_std::{
	convert::{TryFrom, TryInto},
	fmt::Debug,
	marker, result,
};

mod mock;
mod tests;

pub use module::*;

#[frame_support::pallet]
pub mod module {
	use super::*;

	/// Expected price oracle interface. `fetch_price` must return the amount of SettCurrency exchanged for the tracked value.
	pub trait FetchPrice<CurrencyId> {
		/// Fetch the current price.
		fn fetch_price() -> CurrencyId;
	}

	pub trait WeightInfo {
		fn transfer_non_native_currency() -> Weight;
		fn transfer_native_currency() -> Weight;
		fn update_balance_non_native_currency() -> Weight;
		fn update_balance_native_currency_creating() -> Weight;
		fn update_balance_native_currency_killing() -> Weight;
	}

	pub(crate) type BalanceOf<T> = 
		<<T as Config>::SettCurrency as SettCurrency<<T as frame_system::Config>::AccountId>>::Balance;
	pub(crate) type CurrencyIdOf<T> =
		<<T as Config>::SettCurrency as SettCurrency<<T as frame_system::Config>::AccountId>>::CurrencyId;
	pub(crate) type AmountOf<T> =
		<<T as Config>::SettCurrency as ExtendedSettCurrency<<T as frame_system::Config>::AccountId>>::Amount;

	/// The pallet's configuration trait.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The overarching event type.
		type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;
		
		/// The amount of SettCurrency necessary to buy the tracked value. (e.g., 1_100 for 1$)
		type SettCurrencyPrice: FetchPrice<CurrencyId>;

		/// The amount of SettCurrency that are meant to track the value. Example: A value of 1_000 when tracking
		/// Dollars means that the SettCurrency will try to maintain a price of 1_000 SettCurrency for 1$.
		type BaseUnit: Get<CurrencyId>;
		
		type SettCurrency: MergeAccount<Self::AccountId>
			+ ExtendedSettCurrency<Self::AccountId>
			+ LockableSettCurrency<Self::AccountId>
			+ ReservableSettCurrency<Self::AccountId>;

		type NativeCurrency: BasicCurrencyExtended<Self::AccountId, Balance = BalanceOf<Self>, Amount = AmountOf<Self>>
			+ BasicLockableCurrency<Self::AccountId, Balance = BalanceOf<Self>>
			+ BasicReservableCurrency<Self::AccountId, Balance = BalanceOf<Self>>;

		#[pallet::constant]
		type GetNativeCurrencyId: Get<CurrencyIdOf<Self>>;

		/// The initial supply of SettCurrency.
		type InitialSupply: Get<CurrencyId>;
		
		/// The minimum amount of SettCurrency in circulation.
		/// Must be lower than `InitialSupply`.
		type MinimumSupply: Get<CurrencyId>;

		/// Weight information for extrinsics in this module.
		type WeightInfo: WeightInfo;
	}

	#[pallet::error]
	/// Error for stp258 module.
	pub enum Error<T> {
		/// Unable to convert the Amount type into Balance.
		AmountIntoBalanceFailed,
		/// Balance is too low.
		BalanceTooLow,
		/// While trying to increase the balance for an account, it overflowed.
		BalanceOverflow,
		/// An arithmetic operation caused an overflow.
		GenericOverflow,
		/// An arithmetic operation caused an underflow.
		GenericUnderflow,
		/// While trying to increase the Supply, it overflowed.
		SettCurrencySupplyOverflow,
		/// While trying to increase the Supply, it overflowed.
		SettCurrencySupplyUnderflow,
		/// While trying to increase the Supply, it overflowed.
		SupplyOverflow,
		/// Something went very wrong and the price of the currency is zero.
		ZeroPrice,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Currency transfer success. [currency_id, from, to, amount]
		Transferred(CurrencyIdOf<T>, T::AccountId, T::AccountId, BalanceOf<T>),
		/// Update balance success. [currency_id, who, amount]
		BalanceUpdated(CurrencyIdOf<T>, T::AccountId, AmountOf<T>),
		/// Burn success, [currency_id, who, amount]
		Burned(CurrencyIdOf<T>, T::AccountId, AmountOf<T>),
		/// Asset Burn success, [currency_id, who, amount]
		BurnedAsset(CurrencyIdOf<T>, T::AccountId, AmountOf<T>),
		/// Deposit success. [currency_id, who, amount]
		Deposited(CurrencyIdOf<T>, T::AccountId, AmountOf<T>),
		/// Mint success, [currency_id, who, amount]
		Minted(CurrencyIdOf<T>, T::AccountId, AmountOf<T>),
		/// Asset Mint success, [currency_id, who, amount]
		MintedAsset(CurrencyIdOf<T>, T::AccountId, AmountOf<T>),
		/// Withdraw success. [currency_id, who, amount]
		Withdrawn(CurrencyIdOf<T>, T::AccountId, AmountOf<T>),
		/// Some balance was reserved (moved from free to reserved). \[who, value\]
		Reserved(T::AccountId, T::Balance),
		/// Some balance was unreserved (moved from reserved to free). \[who, value\]
		Unreserved(T::AccountId, T::Balance),
		/// Some balance was moved from the reserve of the first account to the second account.
		/// Final argument indicates the destination balance type.
		/// \[from, to, balance, destination_status\]
		ReserveRepatriated(T::AccountId, T::AccountId, T::Balance, Status),
	}

	#[pallet::storage]
	/// The total amount of SettCurrency in circulation.
	#[pallet::getter(fn settcurrency_supply): Get<CurrencyId> = 0]
	pub type SettCurrencySupply<T: Config> = 
			StorageValue<_, CurrencyIdOf<T>, AmountOf<T>, ValueQuery>;

	#[pallet::pallet]
	pub struct Pallet<T>(PhantomData<T>);
	
	#[pallet::hooks]
	impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// The pallet's dispatchable functions.
		const NativeCurrencyId: CurrencyIdOf<T> = T::GetNativeCurrencyId::get();
		const ReserveAsset: CurrencyIdOf<T> = T::GetNativeCurrencyId::get();

		/// The amount of SettCurrencys that represent 1 external value (e.g., 1$).
		const BaseUnit: CurrencyIdOf<T> = T::BaseUnit::get();

		/// The minimum amount of SettCurrency that will be in circulation.
		const MinimumSupply: CurrencyIdOf<T> = T::MinimumSupply::get();
		
		fn deposit_event() = default;

		/// Transfer some balance to another account under `currency_id`.
		///
		/// The dispatch origin for this call must be `Signed` by the
		/// transactor.
		#[pallet::weight(T::WeightInfo::transfer_non_native_currency())]
		pub fn transfer(
			origin: OriginFor<T>,
			dest: <T::Lookup as StaticLookup>::Source,
			currency_id: CurrencyIdOf<T>,
			#[pallet::compact] amount: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			let from = ensure_signed(origin)?;
			let to = T::Lookup::lookup(dest)?;
			<Self as SettCurrency<T::AccountId>>::transfer(currency_id, &from, &to, amount)?;
			Ok(().into())
		}

		/// Transfer some native currency to another account.
		///
		/// The dispatch origin for this call must be `Signed` by the
		/// transactor.
		#[pallet::weight(T::WeightInfo::transfer_native_currency())]
		pub fn transfer_native_currency(
			origin: OriginFor<T>,
			dest: <T::Lookup as StaticLookup>::Source,
			#[pallet::compact] amount: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			let from = ensure_signed(origin)?;
			let to = T::Lookup::lookup(dest)?;
			T::NativeCurrency::transfer(&from, &to, amount)?;

			Self::deposit_event(Event::Transferred(T::GetNativeCurrencyId::get(), from, to, amount));
			Ok(().into())
		}

		/// update amount of account `who` under `currency_id`.
		///
		/// The dispatch origin of this call must be _Root_.
		#[pallet::weight(T::WeightInfo::update_balance_non_native_currency())]
		pub fn update_balance(
			origin: OriginFor<T>,
			who: <T::Lookup as StaticLookup>::Source,
			currency_id: CurrencyIdOf<T>,
			amount: AmountOf<T>,
		) -> DispatchResultWithPostInfo {
			ensure_root(origin)?;
			let dest = T::Lookup::lookup(who)?;
				<Self as ExtendedSettCurrency<T::AccountId>>::update_balance(currency_id, &dest, amount)?;
			Ok(().into())
		}
	}
}

impl<T: Config> SettCurrency<T::AccountId> for Pallet<T> {
	type CurrencyId = CurrencyIdOf<T>;
	type Balance = BalanceOf<T>;

	fn minimum_balance(currency_id: Self::CurrencyId) -> Self::Balance {
		if currency_id == T::GetNativeCurrencyId::get() {
			T::NativeCurrency::minimum_balance()
		} else {
			T::SettCurrency::minimum_balance(currency_id)
		}
	}

	/// The minimum amount of SettCurrency in circulation.
	/// Must be lower than `InitialSupply`.
	/// Cannot set minimum supply for `NativeCurrency`.
	fn minimum_supply(currency_id: Self::CurrencyId) -> Self::Balance {
		if currency_id == T::GetNativeCurrencyId::get() {
			debug::warn!("Cannot set minimum supply for NativeCurrency: {}", currency_id);
            return Err(http::Error::Unknown);
		} else {
			T::SettCurrency::minimum_supply(currency_id)
		}
	}

	fn initial_supply(currency_id: Self::CurrencyId) -> Self::Balance {
		if currency_id == T::GetNativeCurrencyId::get() {
			T::NativeCurrency::initial_supply()
		} else {
			T::SettCurrency::initial_supply(currency_id)
		}
	}

	fn total_issuance(currency_id: Self::CurrencyId) -> Self::Balance {
		if currency_id == T::GetNativeCurrencyId::get() {
			T::NativeCurrency::total_issuance()
		} else {
			T::SettCurrency::total_issuance(currency_id)
		}
	}

	fn total_balance(currency_id: Self::CurrencyId, who: &T::AccountId) -> Self::Balance {
		if currency_id == T::GetNativeCurrencyId::get() {
			T::NativeCurrency::total_balance(who)
		} else {
			T::SettCurrency::total_balance(currency_id, who)
		}
	}

	fn free_balance(currency_id: Self::CurrencyId, who: &T::AccountId) -> Self::Balance {
		if currency_id == T::GetNativeCurrencyId::get() {
			T::NativeCurrency::free_balance(who)
		} else {
			T::SettCurrency::free_balance(currency_id, who)
		}
	}

	fn ensure_can_withdraw(currency_id: Self::CurrencyId, who: &T::AccountId, amount: Self::Balance) -> DispatchResult {
		if currency_id == T::GetNativeCurrencyId::get() {
			T::NativeCurrency::ensure_can_withdraw(who, amount)
		} else {
			T::SettCurrency::ensure_can_withdraw(currency_id, who, amount)
		}
	}

	fn transfer(
		currency_id: Self::CurrencyId,
		from: &T::AccountId,
		to: &T::AccountId,
		amount: Self::Balance,
	) -> DispatchResult {
		if amount.is_zero() || from == to {
			return Ok(());
		}
		if currency_id == T::GetNativeCurrencyId::get() {
			T::NativeCurrency::transfer(from, to, amount)?;
		} else {
			T::SettCurrency::transfer(currency_id, from, to, amount)?;
		}
		Self::deposit_event(RawEvent::Transferred(currency_id, from.clone(), to.clone(), amount));
		Ok(())
	}

	fn deposit(currency_id: Self::CurrencyId, who: &T::AccountId, amount: Self::Balance) -> DispatchResult {
		if amount.is_zero() {
			return Ok(());
		}
		if currency_id == T::GetNativeCurrencyId::get() {
			T::NativeCurrency::deposit(who, amount)?;
		} else {
			T::SettCurrency::deposit(currency_id, who, amount)?;
		}
		Self::deposit_event(RawEvent::Deposited(currency_id, who.clone(), amount));
		Ok(())
	}

	fn withdraw(currency_id: Self::CurrencyId, who: &T::AccountId, amount: Self::Balance) -> DispatchResult {
		if amount.is_zero() {
			return Ok(());
		}
		if currency_id == T::GetNativeCurrencyId::get() {
			T::NativeCurrency::withdraw(who, amount)?;
		} else {
			T::SettCurrency::withdraw(currency_id, who, amount)?;
		}
		Self::deposit_event(RawEvent::Withdrawn(currency_id, who.clone(), amount));
		Ok(())
	}

	fn can_slash(currency_id: Self::CurrencyId, who: &T::AccountId, amount: Self::Balance) -> bool {
		if currency_id == T::GetNativeCurrencyId::get() {
			T::NativeCurrency::can_slash(who, amount)
		} else {
			T::SettCurrency::can_slash(currency_id, who, amount)
		}
	}

	fn slash(currency_id: Self::CurrencyId, who: &T::AccountId, amount: Self::Balance) -> Self::Balance {
		if currency_id == T::GetNativeCurrencyId::get() {
			T::NativeCurrency::slash(who, amount)
		} else {
			T::SettCurrency::slash(currency_id, who, amount)
		}
	}

	fn mint(currency_id: Self::CurrencyId, who: &T::AccountId, amount: Self::Balance) -> result::Result<(), &'static str>{
		if currency_id == T::GetNativeCurrencyId::get() {
			T::NativeCurrency::mint(who, amount)
		} else {
			T::SettCurrency::mint(currency_id, who, amount)
		}
	}

	fn burn(currency_id: Self::CurrencyId, who: &T::AccountId, amount: Self::Balance) -> result::Result<(), &'static str>{
		if currency_id == T::GetNativeCurrencyId::get() {
			T::NativeCurrency::burn(who, amount)
		} else {
			T::SettCurrency::burn(currency_id, who, amount)
		}
	}

	/// The Pegs of the BasketToken/BasketSettCurrency.
	/// To form the BasketToken based on the `Price` of the - 
	/// respective currencies in peg position.
	/// The `NativeCurrency` Cannot Be `BasketSettCurrency`.
	fn basket_token(
		currency_id: Self::CurrencyId,
		peg_price: FetchPrice<CurrencyId>,
	) -> DispatchResult {
		if currency_id == T::GetNativeCurrencyId::get() {
			debug::warn!("The Native Currency: {} Cannot Be BasketTokenSettCurrency", currency_id);
            return Err(http::Error::Unknown);
		} else {
			T::SettCurrency::basket_token(peg_price, currency_id)
		}
	}
}

impl<T: Config> ExtendedSettCurrency<T::AccountId> for Pallet<T> {
	type Amount = AmountOf<T>;

	fn update_balance(currency_id: Self::CurrencyId, who: &T::AccountId, by_amount: Self::Amount) -> DispatchResult {
		if currency_id == T::GetNativeCurrencyId::get() {
			T::NativeCurrency::update_balance(who, by_amount)?;
		} else {
			T::SettCurrency::update_balance(currency_id, who, by_amount)?;
		}
		Self::deposit_event(RawEvent::BalanceUpdated(currency_id, who.clone(), by_amount));
		Ok(())
	}
}

impl<T: Config> LockableSettCurrency<T::AccountId> for Pallet<T> {
	type Moment = T::BlockNumber;

	fn set_lock(
		lock_id: LockIdentifier,
		currency_id: Self::CurrencyId,Pallet
		who: &T::AccountId,
		amount: Self::Balance,
	) -> DispatchResult {
		if currency_id == T::GetNativeCurrencyId::get() {
			T::NativeCurrency::set_lock(lock_id, who, amount)
		} else {
			T::SettCurrency::set_lock(lock_id, currency_id, who, amount)
		}
	}

	fn extend_lock(
		lock_id: LockIdentifier,
		currency_id: Self::CurrencyId,
		who: &T::AccountId,
		amount: Self::Balance,
	) -> DispatchResult {
		if currency_id == T::GetNativeCurrencyId::get() {
			T::NativeCurrency::extend_lock(lock_id, who, amount)
		} else {
			T::SettCurrency::extend_lock(lock_id, currency_id, who, amount)
		}
	}

	fn remove_lock(lock_id: LockIdentifier, currency_id: Self::CurrencyId, who: &T::AccountId) -> DispatchResult {
		if currency_id == T::GetNativeCurrencyId::get() {
			T::NativeCurrency::remove_lock(lock_id, who)
		} else {
			T::SettCurrency::remove_lock(lock_id, currency_id, who)
		}
	}
}

impl<T: Config> ReservableSettCurrency<T::AccountId> for Pallet<T> {
	fn can_reserve(currency_id: Self::CurrencyId, who: &T::AccountId, value: Self::Balance) -> bool {
		if currency_id == T::GetNativeCurrencyId::get() {
			T::NativeCurrency::can_reserve(who, value)
		} else {
			T::SettCurrency::can_reserve(currency_id, who, value)
		}
	}

	fn slash_reserved(currency_id: Self::CurrencyId, who: &T::AccountId, value: Self::Balance) -> Self::Balance {
		if currency_id == T::GetNativeCurrencyId::get() {
			T::NativeCurrency::slash_reserved(who, value)
		} else {
			T::SettCurrency::slash_reserved(currency_id, who, value)
		}
	}
	
	fn mint(currency_id: Self::CurrencyId, who: &T::AccountId, value: Self::Balance) -> result::Result<(), &'static str>{
		if currency_id == T::GetNativeCurrencyId::get() {
			T::NativeCurrency::mint(who, value)
		} else {
			T::SettCurrency::mint(currency_id, who, value)
		}
	}

	fn burn(currency_id: Self::CurrencyId, who: &T::AccountId, value: Self::Balance) -> result::Result<(), &'static str>{
		if currency_id == T::GetNativeCurrencyId::get() {
			T::NativeCurrency::burn(who, value)
		} else {
			T::SettCurrency::burn(currency_id, who,value)
		}
	}

	fn reserved_balance(currency_id: Self::CurrencyId, who: &T::AccountId) -> Self::Balance {
		if currency_id == T::GetNativeCurrencyId::get() {
			T::NativeCurrency::reserved_balance(who)
		} else {
			T::SettCurrency::reserved_balance(currency_id, who)
		}
	}

	fn reserve(currency_id: Self::CurrencyId, who: &T::AccountId, value: Self::Balance) -> DispatchResult {
		if currency_id == T::GetNativeCurrencyId::get() {
			T::NativeCurrency::reserve(who, value)
		} else {
			T::SettCurrency::reserve(currency_id, who, value)
		}
	}

	fn unreserve(currency_id: Self::CurrencyId, who: &T::AccountId, value: Self::Balance) -> Self::Balance {
		if currency_id == T::GetNativeCurrencyId::get() {
			T::NativeCurrency::unreserve(who, value)
		} else {
			T::SettCurrency::unreserve(currency_id, who, value)
		}
	}

	fn repatriate_reserved(
		currency_id: Self::CurrencyId,
		slashed: &T::AccountId,
		beneficiary: &T::AccountId,
		value: Self::Balance,
		status: BalanceStatus,
	) -> result::Result<Self::Balance, DispatchError> {
		if currency_id == T::GetNativeCurrencyId::get() {
			T::NativeCurrency::repatriate_reserved(slashed, beneficiary, value, status)
		} else {
			T::SettCurrency::repatriate_reserved(currency_id, slashed, beneficiary, value, status)
		}
	}
}

pub struct SettCurrency<T, GetCurrencyId>(marker::PhantomData<T>, marker::PhantomData<GetCurrencyId>);

impl<T, GetCurrencyId> BasicCurrency<T::AccountId> for Currency<T, GetCurrencyId>
where
	T: Config,
	GetCurrencyId: Get<CurrencyIdOf<T>>,
{
	type Balance = BalanceOf<T>;

	fn minimum_balance() -> Self::Balance {
		<Pallet<T>>::minimum_balance(GetCurrencyId::get())
	}

	fn total_issuance() -> Self::Balance {
		<Pallet<T>>::total_issuance(GetCurrencyId::get())
	}

	fn total_balance(who: &T::AccountId) -> Self::Balance {
		<Pallet<T>>::total_balance(GetCurrencyId::get(), who)
	}

	fn free_balance(who: &T::AccountId) -> Self::Balance {
		<Pallet<T>>::free_balance(GetCurrencyId::get(), who)
	}

	fn ensure_can_withdraw(who: &T::AccountId, amount: Self::Balance) -> DispatchResult {
		<Pallet<T>>::ensure_can_withdraw(GetCurrencyId::get(), who, amount)
	}

	fn transfer(from: &T::AccountId, to: &T::AccountId, amount: Self::Balance) -> DispatchResult {
		<Pallet<T> as SettCurrency<T::AccountId>>::transfer(GetCurrencyId::get(), from, to, amount)
	}

	fn deposit(who: &T::AccountId, amount: Self::Balance) -> DispatchResult {
		<Pallet<T>>::deposit(GetCurrencyId::get(), who, amount)
	}

	fn withdraw(who: &T::AccountId, amount: Self::Balance) -> DispatchResult {
		<Pallet<T>>::withdraw(GetCurrencyId::get(), who, amount)
	}

	fn can_slash(who: &T::AccountId, amount: Self::Balance) -> bool {
		<Pallet<T>>::can_slash(GetCurrencyId::get(), who, amount)
	}

	fn slash(who: &T::AccountId, amount: Self::Balance) -> Self::Balance {
		<Pallet<T>>::slash(GetCurrencyId::get(), who, amount)
	}
	
	fn mint(who: &T::AccountId, amount: Self::Balance) -> result::Result<(), &'static str>{
		<Pallet<T>>::mint(GetCurrencyId::get(), who, amount)
	}

	fn burn(who: &T::AccountId, amount: Self::Balance) -> result::Result<(), &'static str>{
		<Pallet<T>>::burn(GetCurrencyId::get(), who, amount)
	}
}

impl<T, GetCurrencyId> BasicCurrencyExtended<T::AccountId> for SettCurrency<T, GetCurrencyId>
where
	T: Config,
	GetCurrencyId: Get<CurrencyIdOf<T>>,
{
	type Amount = AmountOf<T>;

	fn update_balance(who: &T::AccountId, by_amount: Self::Amount) -> DispatchResult {
		<Pallet<T> as ExtendedSettCurrency<T::AccountId>>::update_balance(GetCurrencyId::get(), who, by_amount)
	}
}

impl<T, GetCurrencyId> BasicLockableCurrency<T::AccountId> for Currency<T, GetCurrencyId>
where
	T: Config,
	GetCurrencyId: Get<CurrencyIdOf<T>>,
{
	type Moment = T::BlockNumber;

	fn set_lock(lock_id: LockIdentifier, who: &T::AccountId, amount: Self::Balance) -> DispatchResult {
		<Pallet<T> as LocPalletkableSettCurrency<T::AccountId>>::set_lock(lock_id, GetCurrencyId::get(), who, amount)
	}

	fn extend_lock(lock_id: LockIdentifier, who: &T::AccountId, amount: Self::Balance) -> DispatchResult {
		<Pallet<T> as LockableSettCurrency<T::AccountId>>::extend_lock(lock_id, GetCurrencyId::get(), who, amount)
	}

	fn remove_lock(lock_id: LockIdentifier, who: &T::AccountId) -> DispatchResult {
		<Pallet<T> as LockableSettCurrency<T::AccountId>>::remove_lock(lock_id, GetCurrencyId::get(), who)
	}
}

impl<T, GetCurrencyId> BasicReservableCurrency<T::AccountId> for Currency<T, GetCurrencyId>
where
	T: Config,
	GetCurrencyId: Get<CurrencyIdOf<T>>,
{
	fn can_reserve(who: &T::AccountId, value: Self::Balance) -> bool {
		<Pallet<T> as ReservableSettCurrency<T::AccountId>>::can_reserve(GetCurrencyId::get(), who, value)
	}

	fn slash_reserved(who: &T::AccountId, value: Self::Balance) -> Self::Balance {
		<Pallet<T> as ReservableSettCurrency<T::AccountId>>::slash_reserved(GetCurrencyId::get(), who, value)
	}
	
	fn mint(who: &T::AccountId, value: Self::Balance) -> result::Result<(), &'static str>{
		<Pallet<T> as ReservableSettCurrency<TPallet::AccountId>>::mint(GetCurrencyId::get(), who, value)
	}

	fn burn(who: &T::AccountId, value: Self::Balance) -> result::Result<(), &'static str>{
		<Pallet<T> as ReservableSettCurrency<T::AccountId>>::burn(GetCurrencyId::get(), who, value)
	}

	fn reserved_balance(who: &T::AccountId) -> Self::Balance {
		<Pallet<T> as ReservableSettCurrency<T::AccountId>>::reserved_balance(GetCurrencyId::get(), who)
	}

	fn reserve(who: &T::AccountId, value: Self::Balance) -> DispatchResult {
		<Pallet<T> as ReservableSettCurrency<T::AccountId>>::reserve(GetCurrencyId::get(), who, value)
	}

	fn unreserve(who: &T::AccountId, value: Self::Balance) -> Self::Balance {
		<Pallet<T> as ReservableSettCurrency<T::AccountId>>::unreserve(GetCurrencyId::get(), who, value)
	}

	fn repatriate_reserved(
		slashed: &T::AccountId,
		beneficiary: &T::AccountId,
		value: Self::Balance,
		status: BalanceStatus,
	) -> result::Result<Self::Balance, DispatchError> {
		<Pallet<T> as ReservableSettCurrency<T::AccountId>>::repatriate_reserved(
			GetCurrencyId::get(),
			slashed,
			beneficiary,
			value,
			status,
		)
	}
}

pub type NativeCurrencyOf<T> = Currency<T, <T as Config>::GetNativeCurrencyId>;

/// Adapt other currency traits implementation to `BasicCurrency`.
pub struct BasicCurrencyAdapter<T, Currency, Amount, Moment>(marker::PhantomData<(T, Currency, Amount, Moment)>);

type PalletBalanceOf<A, Currency> = <Currency as PalletCurrency<A>>::Balance;

// Adapt `frame_support::traits::Currency`
impl<T, AccountId, Currency, Amount, Moment> BasicCurrency<AccountId>
	for BasicCurrencyAdapter<T, Currency, Amount, Moment>
where
	Currency: PalletCurrency<AccountId>,
	T: Config,
{
	type Balance = PalletBalanceOf<AccountId, Currency>;

	fn minimum_balance() -> Self::Balance {
		Currency::minimum_balance()
	}

	fn total_issuance() -> Self::Balance {
		Currency::total_issuance()
	}

	fn total_balance(who: &AccountId) -> Self::Balance {
		Currency::total_balance(who)
	}

	fn free_balance(who: &AccountId) -> Self::Balance {
		Currency::free_balance(who)
	}

	fn ensure_can_withdraw(who: &AccountId, amount: Self::Balance) -> DispatchResult {
		let new_balance = Self::free_balance(who)
			.checked_sub(&amount)
			.ok_or(Error::<T>::BalanceTooLow)?;

		Currency::ensure_can_withdraw(who, amount, WithdrawReasons::all(), new_balance)
	}

	fn transfer(from: &AccountId, to: &AccountId, amount: Self::Balance) -> DispatchResult {
		Currency::transfer(from, to, amount, ExistenceRequirement::AllowDeath)
	}

	fn deposit(who: &AccountId, amount: Self::Balance) -> DispatchResult {
		let _ = Currency::deposit_creating(who, amount);
		Ok(())
	}

	fn withdraw(who: &AccountId, amount: Self::Balance) -> DispatchResult {
		Currency::withdraw(who, amount, WithdrawReasons::all(), ExistenceRequirement::AllowDeath).map(|_| ())
	}

	fn can_slash(who: &AccountId, amount: Self::Balance) -> bool {
		Currency::can_slash(who, amount)
	}

	fn slash(who: &AccountId, amount: Self::Balance) -> Self::Balance {
		let (_, gap) = Currency::slash(who, amount);
		gap
	}

	fn mint(who: &AccountId, amount: Self::Balance,) -> result::Result<(), &'static str>{
		Currency::mint(who, amount)
	}

	fn burn(who: &AccountId, amount: Self::Balance,) -> result::Result<(), &'static str>{
		Currency::burn(who, amount)
	}
}

// Adapt `frame_support::traits::Currency`
impl<T, AccountId, Currency, Amount, Moment> BasicCurrencyExtended<AccountId>
	for BasicCurrencyAdapter<T, Currency, Amount, Moment>
where
	Amount: Signed
		+ TryInto<PalletBalanceOf<AccountId, Currency>>
		+ TryFrom<PalletBalanceOf<AccountId, Currency>>
		+ SimpleArithmetic
		+ Codec
		+ Copy
		+ MaybeSerializeDeserialize
		+ Debug
		+ Default,
	Currency: PalletCurrency<AccountId>,
	T: Config,
{
	type Amount = Amount;

	fn update_balance(who: &AccountId, by_amount: Self::Amount) -> DispatchResult {
		let by_balance = by_amount
			.abs()
			.try_into()
			.map_err(|_| Error::<T>::AmountIntoBalanceFailed)?;
		if by_amount.is_positive() {
			Self::deposit(who, by_balance)
		} else {
			Self::withdraw(who, by_balance)
		}
	}
}

// Adapt `frame_support::traits::LockableCurrency`
impl<T, AccountId, Currency, Amount, Moment> BasicLockableCurrency<AccountId>
	for BasicCurrencyAdapter<T, Currency, Amount, Moment>
where
	Currency: PalletLockableCurrency<AccountId>,
	T: Config,
{
	type Moment = Moment;

	fn set_lock(lock_id: LockIdentifier, who: &AccountId, amount: Self::Balance) -> DispatchResult {
		Currency::set_lock(lock_id, who, amount, WithdrawReasons::all());
		Ok(())
	}

	fn extend_lock(lock_id: LockIdentifier, who: &AccountId, amount: Self::Balance) -> DispatchResult {
		Currency::extend_lock(lock_id, who, amount, WithdrawReasons::all());
		Ok(())
	}

	fn remove_lock(lock_id: LockIdentifier, who: &AccountId) -> DispatchResult {
		Currency::remove_lock(lock_id, who);
		Ok(())
	}
}

// Adapt `frame_support::traits::ReservableCurrency`
impl<T, AccountId, Currency, Amount, Moment> BasicReservableCurrency<AccountId>
	for BasicCurrencyAdapter<T, Currency, Amount, Moment>
where
	Currency: PalletReservableCurrency<AccountId>,
	T: Config,
{
	fn can_reserve(who: &AccountId, value: Self::Balance) -> bool {
		Currency::can_reserve(who, value)
	}

	fn slash_reserved(who: &AccountId, value: Self::Balance) -> Self::Balance {
		let (_, gap) = Currency::slash_reserved(who, value);
		gap
	}

	fn mint(who: &AccountId, value: Self::Balance,) -> result::Result<(), &'static str>{
		Currency::mint(who, value)
	}

	fn burn(who: &AccountId, value: Self::Balance,) -> result::Result<(), &'static str>{
		Currency::burn(who, value)
	}

	fn reserved_balance(who: &AccountId) -> Self::Balance {
		Currency::reserved_balance(who)
	}

	fn reserve(who: &AccountId, value: Self::Balance) -> DispatchResult {
		Currency::reserve(who, value)
	}

	fn unreserve(who: &AccountId, value: Self::Balance) -> Self::Balance {
		Currency::unreserve(who, value)
	}

	fn repatriate_reserved(
		slashed: &AccountId,
		beneficiary: &AccountId,
		value: Self::Balance,
		status: BalanceStatus,
	) -> result::Result<Self::Balance, DispatchError> {
		Currency::repatriate_reserved(slashed, beneficiary, value, status)
	}
}

impl<T: Config> MergeAccount<T::AccountId> for Pallet<T> {
	fn merge_account(source: &T::AccountId, dest: &T::AccountId) -> DispatchResult {
		with_transaction_result(|| {
			// transfer non-native stablecoin free to dest
			T::SettCurrency::merge_account(source, dest)?;

			// unreserve all reserved currency
			T::NativeCurrency::unreserve(source, T::NativeCurrency::reserved_balance(source));

			// transfer all free to dest
			T::NativeCurrency::transfer(source, dest, T::NativeCurrency::free_balance(source))
		})
	}
}
