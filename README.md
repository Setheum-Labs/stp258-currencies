 # Setheum Tokenization Protocol 258 Standard
 ## Setheum Tokenization Protocol 258 Standard
 Multi-Currency Stablecoin SERP Module


 ## Overview

 The STP258 Standard module provides a mixed stablecoin system, by configuring a
 native currency which implements `Stp258AssetExtended`, and a
 multi-currency which implements `Stp258Currency`.

 ### Implementations

 The stp258 module provides implementations for following traits.

 - `Stp258Currency` - Abstraction over a fungible multi-currency stablecoin system.
 - `Stp258CurrencyExtended` - Extended `Stp258Currency` with additional helper
   types and methods, like updating balance
 by a given signed integer amount.

 ## Interface

 ### Dispatchable Functions

 - `transfer` - Transfer some balance to another account, in a given
   currency.
 - `transfer_native_currency` - Transfer some balance to another account, in
   native currency set in
 `Config::Stp258Native`.
 - `update_balance` - Update balance by signed integer amount, in a given
   currency, root origin required.
 
## Acknowledgement & Reference

This Pallet is inspired by the [ORML Currencies](https://github.com/open-web3-stack/open-runtime-module-library/blob/master/currencies) Pallet developed by [Open Web3 Stack](https://github.com/open-web3-stack/), for reference check [The ORML Repo](https://github.com/open-web3-stack/open-runtime-module-library).

This Pallet is inspired by the [Price](https://github.com/Setheum-Labs/price/) Pallet developed by [Setheum Labs](https://github.com/Setheum-Labs/), for reference on use check [The Setheum-Labs/Price Repo](https://github.com/Setheum-Labs/price/).

This Pallet is inspired by the [Stablecoin](https://github.com/apopiak/stablecoin) Pallet developed by [Alexander Popiak](https://github.com/apopiak), for reference check [The Apopiak/Stablecoin Repo](https://github.com/apopiak/stablecoin).
