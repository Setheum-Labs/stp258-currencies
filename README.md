 # STP-258 Currencies
 ## Setheum Tokenization Protocol 258 Standard
 Multi-Currency Stablecoin SERP Module based on `Stp258Standard` built on top of `Stp258Serp` and `SerpTraits`.

 ## Overview

 The STP258 Currencies module provides a mixed stablecoin system, by configuring a
 native currency which implements `Stp258AssetExtended`, and a
 multi-currency which implements `Stp258Currency`.

 This module is based on the [STP-258 Standard](https://github.com/Setheum-Labs/stp258-standard) built on the [STP-258 Serp](https://github.com/Setheum-Labs/stp258-serp) implementing the [STP-258 Traits](https://github.com/Setheum-Labs/serp-traits).

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
