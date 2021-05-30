Unit testing
============

Refactorings with typing
========================
* `KittyIndex`

transfer()
==========

Hot-deployment
==============

Metadata
========
In polkadot.js.org/#apps/rpc -> state -> getMetadata()
- see extinsic calls together with documentation `/// blah` and parameter names, types

On-chain storage
================
storage hasher:
* twox_64_concat or twox_128_concat - for trusted, eg. u32 known values
* blake2_128_concat - for not trusted, eg. balance
...concat means uses hash+actual_key
...Not understanding the diff between trusted/untrusted


To watch rpc calls
------------------
Open DevTools, click on WSS 
Go to https://polkadot.js.org/apps/#/chainstate -> timestamp -> now()
In DevTools, open WSS session on 127.0.0.1, filter by `state_subscribeStorage`, look for last WSS message (there might be few):
```{
id: 18
jsonrpc: "2.0"
method: "state_subscribeStorage"
params: [["0xf0c365c3cf59d671eb72da0e7a4113c49f1f0515f462cdcf84e0f1d6045dfcbb"]]
}
```
Goto Raw Storage, type in 0xf0c365c3cf59d671eb72da0e7a4113c49f1f0515f462cdcf84e0f1d6045dfcbb, see the value change every 2s (block time)

SCALE codec
===========
Compact, lightweight, not-self encoding (need schema to decode, unlike eg. json)
* xx xx xx 00                         -> 00 used to indicate 1 byte, max value 63
* yL yL yL 01 yH yH yH yH             -> 01 indicates 2 bytes
* yL yL yL 01 yM yM yM yM yH yH yH yH -> 10 indicates 3 bytes


Assignment
==========
Copy from lecture:
* add tests
* add types
* implement transfer()
This week's assignment:
* unit test transfer()
* implement exchange functionality:
  * fn set_price(KittyId, value)
  * fn buy(KittyId, value)
  * errors: KittyPriceTooLow
  * events: KittTransfered, KittySold

QQQ
===
* can extrinsics return anything?