# Setheum Tokenization Protocol 258
Multi-Currency Stablecoin SERP Module

## Overview
  The stp258 module provides a mixed stablecoin system, by configuring a
 native currency which implements `BasicCurrencyExtended`, and a
 multi-currency which implements `SettCurrency`.
 
 It also implement an atomic swap, to atomically swap currencies 
  `create_swap` - called by a sender to register a new atomic swap
  `claim_swap` - called by the target to approve a swap
  `cancel_swap` - may be called by a sender after a specified duration.

 It also implement an price fetch `FetchPrice`, to fetch currency prices. 
  `set_price` - called to manually set currency price.
  `FetchPriceFor` - called from an offchain worker to fetch off-chain price.
  
 It also provides an adapter, to adapt `frame_support::traits::Currency`
 implementations into `BasicCurrencyExtended`.

 The stp258 module provides functionality of both `ExtendedSettCurrency`
 and `BasicCurrencyExtended`, via unified interfaces, and all calls would be
 delegated to the underlying multi-currency and base currency system.
 A native currency ID could be set by `Config::GetNativeCurrencyId`, to
 identify the native currency.

 ### Implementations

 The stp258 module provides implementations for following traits.

 - `SettCurrency` - Abstraction over a fungible multi-currency stablecoin system 
 that includes `basket_token` as pegged to a basket of currencies, `price` of settcurrencies and `sett_swap` to atomically swap currencies.
 - `ExtendedSettCurrency` - Extended `SettCurrency` with additional helper
   types and methods, like updating balance
 by a given signed integer amount.

 ## Interface

 ### Dispatchable Functions
 - `transfer` - Transfer some balance to another account, in a given   currency. - `transfer_native_currency` - Transfer some balance to another account, in   native currency set in `Config::NativeCurrency`. - `update_balance` - Update balance by signed integer amount, in a given  currency, root origin required.

 - `mint` - Mint some amount to some given account, in a given
   currency.
   
## Acknowledgement & Reference

This Pallet is inspired by the [Atomic Swap](https://github.com/Setheum-Labs/price/) Pallet developed by [Parity Tech](https://github.com/paritytech/) as a part of [Substrate FRAME](https://github.com/paritytech/substrate/tree/master/frame), for reference on use check [The paritytech/substrate Repo](https://github.com/paritytech/substrate).

This Pallet is inspired by the [ORML Currencies](https://github.com/open-web3-stack/open-runtime-module-library/blob/master/currencies) Pallet developed by [Open Web3 Stack](https://github.com/open-web3-stack/), for reference check [The ORML Repo](https://github.com/open-web3-stack/open-runtime-module-library).

This Pallet is inspired by the [Price](https://github.com/Setheum-Labs/price/) Pallet developed by [Setheum Labs](https://github.com/Setheum-Labs/), for reference on use check [The Setheum-Labs/Price Repo](https://github.com/Setheum-Labs/price/).

This Pallet is inspired by the [Stablecoin](https://github.com/apopiak/stablecoin) Pallet developed by [Alexander Popiak](https://github.com/apopiak), for reference check [The Apopiak/Stablecoin Repo](https://github.com/apopiak/stablecoin).
