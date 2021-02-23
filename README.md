# Setheum Tokenization Protocol 258
Multi-Currency Stablecoin SERP Module

## Overview


 The stp258 module provides a mixed stablecoin system, by configuring a
 native currency which implements `ExtendedBasicCurrency`, and a
 multi-currency which implements `SettCurrency`.

 It also provides an adapter, to adapt `frame_support::traits::Currency`
 implementations into `ExtendedBasicCurrency`.

 The stp258 module provides functionality of both `ExtendedSettCurrency`
 and `ExtendedBasicCurrency`, via unified interfaces, and all calls would be
 delegated to the underlying multi-currency and base currency system.
 A native currency ID could be set by `Config::GetNativeCurrencyId`, to
 identify the native currency.

 ### Implementations

 The stp258 module provides implementations for following traits.

 - `SettCurrency` - Abstraction over a fungible multi-currency stablecoin system.
 - `ExtendedSettCurrency` - Extended `SettCurrency` wih additional helper
  types and methods, like updating balance
 by a given signed integer amount.

 ## Interface

 ### Dispatchable Functions

 - `transfer` - Transfer some balance to another account, in a given
   currency.
 - `transfer_native_currency` - Transfer some balance to another account, in
   native currency set in
 `Config::NativeCurrency`.
 - `update_balance` - Update balance by signed integer amount, in a given
   currency, root origin required.

## Acknowledgement

This Pallet is inspired by the [ORML Currencies](https://github.com/open-web3-stack/open-runtime-module-library/blob/master/currencies) Pallet originally developed by [Open Web3 Stack](https://github.com/open-web3-stack/), for reference check [The ORML Repo](https://github.com/open-web3-stack/open-runtime-module-library).

This Pallet is inspired by the [Stablecoin](https://github.com/apopiak/stablecoin) Pallet originally developed by [Alexander Popiak](https://github.com/apopiak), for reference check [The Apopiak/Stablecoin Repo](https://github.com/apopiak/stablecoin).
