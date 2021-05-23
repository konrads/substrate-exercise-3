Hints, explanations, notes, etc
===============================

To build:
```
./scripts/init.sh
cargo build
```

To run built
```
./target/debug/node-template --dev --tmp -lruntime=debug
```

To expand macros, useful for understanding macro internals.
Note, still getting a huge entry for `pub const WASM_BINARY : Option < & [u8] > `
```
cargo install cargo-expand
SKIP_WASM_BUILD= cargo expand -p node-template-runtime > runtime.rs  # generates top level runtime.rs
SKIP_WASM_BUILD= cargo expand -p pallet-template > template.rs       # generates top level template.rs
# old way of skipping WASM binary generation
# BUILD_DUMMY_WASM_BINARY= cargo expand -p node-template-runtime > runtime.rs  # generates top level runtime.rs
# BUILD_DUMMY_WASM_BINARY= cargo expand -p pallet-template > template.rs       # generates top level template.rs
```

Kitties pallet design
---------------------
* Calls
  * create
* Storages
  * Kitties: double_map AccountId, u32 => Option<Kitty>
  * NextKittyId: u32
* Types
  * struct Kitty([u8; 16])
* Events
  * KittyCreated
    * owner: AccountId
    * kitty_id: u32
    * kitty: Kitty

Type Definitions
----------------
Start node via:
```
./target/debug/node-template --dev --tmp -lruntime=debug
```
Go to: https://polkadot.js.org/apps/#/explorer
Ensure local dev is selected: Development -> Local Node -> 127.0.0.1:9944
Then go to Settings -> Developer

example:
```
type Index = u32;

type Tuple = (u32, u64);

struct Foo {
  field: u32,
  second_field: Vec<Index>,
}

struct Bar(u32, u64);

enum TestEnum {
  First,
  Second,
  Third
}

enum AnotherEnum {
  NoArgs,
  WithOneArg(u32),
  WithMoreArgs(Option<u32>, u32),
}

```
...translate to:
{
  "Index": "u32",
  "Tuple": "(u32, u64)",
  "Foo": {
    "field": "u32",
    "secondField": "Vec<Index>"
  },
  "Bar": "(u32, u64)",
  "TestEnum": {
    "_enum": ["First", "Second", "Third"]
  },
  "AnotherEnum": {
    "_enum": {
      "NoArgs": "Null",
      "WithOneArg": "u32",
      "WithMoreArgs": "(Option<u32>, u32)"
    }
  }
}
```

actual:
```
pub struct Kitty(pub [u8; 16]);
enum Gender {
    Male,
    Female
};
```

...translate to:
```
{
  "Kitty": "[u8; 16]",
  "Gender": {
    "_enum": ["Male", "Female"]
  },

  "AccountInfo": "AccountInfoWithProviders", -- add if getting: Error: Unable to decode storage system.account: entry 0:: createType(AccountInfo)::...
                                             -- provided by: http://questionhub.mvp.studio/?qa=20427/polkadot-js-javascript-console-error&show=20427
  "AccountInfo": "AccountInfoWithDualRefCount" -- alternative to AccountInfoWithProviders?... not sure the diff
}
```
... consider other types, not yet sure what that means all types: https://github.com/substrate-developer-hub/recipes/blob/master/runtimes/super-runtime/types.json
```
{
  "Address": "AccountId",
  "LookupSource": "AccountId",
  "AccountInfo": "AccountInfoWithDualRefCount",
  "ContinuousAccountData": {
    "principal": "u64",
    "deposit_date": "BlockNumber"
  },
  "U16F16": "[u8; 4]",
  "GroupIndex": "u32",
  "ValueStruct": {
    "integer": "i32",
    "boolean": "bool"
  },
  "BufferIndex": "u8",
  "AccountIdOf": "AccountId",
  "BalanceOf": "Balance",
  "FundInfoOf": "FundInfo",
  "FundInfo": {
    "beneficiary": "AccountId",
    "deposit": "Balance",
    "raised": "Balance",
    "end": "BlockNumber",
    "goal": "Balance"
  },
  "FundIndex": "u32",
  "InnerThing": {
    "number": "u32",
    "hash": "Hash",
    "balance": "Balance"
  },
  "SuperThing": {
    "super_number": "u32",
    "inner_thing": "InnerThing"
  },
  "InnerThingOf": "InnerThing"
}
```


Note: Ensure the change above is picked up in Chrome -> Dev tools -> console!
failure:
```
polkadot.02.ac22f374.js:1 2021-05-20 13:14:21        METADATA: Unknown types found, no types for Kitty
```
vs success:
```
Detected types: Kitty
```

...FIXME: not sure about:
```
DevTools failed to load SourceMap: Could not load content for chrome-extension://ljdobmomdgdljniojadhoplhkpialdid/common/browser-polyfill.js.map: HTTP error: status code 404, net::ERR_UNKNOWN_URL_SCHEME
```

Try out
-------

Go to https://polkadot.js.org/apps/#/extrinsics (Developer -> Extrinsics)
Alice -> kitties -> create() -> Submit transaction -> Sign and Submit

Should see event in Network -> Explorer:
```
kitties.KittyCreated
A kitty is created. [owner, kitty_id, kitty] 
```

Validate chain state:
Developer -> Chain State -> Storage -> kitties -> nextKittyId(): u32
Developer -> Chain State -> Storage -> kitties -> kitties(AccountId, u32): Option<Kitty>
Developer -> Chain State -> Storage -> kitties -> parents(AccountId, u32, u32): Option<(u32, u32)>
...note if click `include option` + `+`, you get a list of kitties. otherwise you get the 1 (first one?)

Execute via JS API
------------------
Go to Developer -> Javascript -> Make transfer and listen to events (to bemodified below)
```
const ALICE = '5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY';

// Create a extrinsic, transferring randomAmount units to Bob.
const create = api.tx.kitties.create();

// Sign and Send the transaction
await create.signAndSend(ALICE, ({ events = [], status }) => {
  if (status.isInBlock) {
    console.log('Successful transfer of create with hash ' + status.asInBlock.toHex());
  } else {
    console.log('Status of transfer: ' + status.type);
  }

  events.forEach(({ phase, event: { data, method, section } }) => {
    console.log(phase.toString() + ' : ' + section + '.' + method + ' ' + data.toString());
  });
});
```