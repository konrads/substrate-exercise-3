use super::*;
use crate as kitties;
use sp_core::H256;
use frame_support::{parameter_types, assert_ok, assert_noop};
use sp_runtime::{
	traits::{BlakeTwo256, IdentityLookup}, testing::Header,
};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Module, Call, Config, Storage, Event<T>},
		Balances: pallet_balances::{Module, Call, Storage, Config<T>, Event<T>},
		KittiesModule: kitties::{Module, Call, Storage, Event<T>},
	}
);

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const SS58Prefix: u8 = 42;
}

impl frame_system::Config for Test {
	type BaseCallFilter = ();
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type Origin = Origin;
	type Call = Call;
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = Event;
	type BlockHashCount = BlockHashCount;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<u64>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = SS58Prefix;
}

parameter_types! {
	pub const ExistentialDeposit: u64 = 1;
}
impl pallet_balances::Config for Test {
	type MaxLocks = ();
	type Balance = u64;
	type Event = Event;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = ();
}

impl Config for Test {
	type Event = Event;
	type Currency = Balances;
	type KittyIndex = u32;
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut t= frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();
	pallet_balances::GenesisConfig::<Test>{
		balances: vec![
			(100, 100),  // me with balance of 100
			(200, 200),  // poor with balance of 200
			(300, 300),  // rich with balance of 300
	]}.assimilate_storage(&mut t).unwrap();
	let mut t: sp_io::TestExternalities = t.into();
    t.execute_with(|| System::set_block_number(1) );
    t
}

fn last_event() -> Option<Event> {
    System::events().last().map(|e| e.event.clone())
}

#[test]
fn can_create_test() {
    new_test_ext().execute_with(|| {
        assert_ok!(KittiesModule::create(Origin::signed(100)));

        let kitty = Kitty([59, 250, 138, 82, 209, 39, 141, 109, 163, 238, 183, 145, 235, 168, 18, 122]);

        assert_eq!(KittiesModule::kitties(100, 0), Some(kitty.clone()));
        assert_eq!(KittiesModule::next_kitty_id(), 1);

        assert_eq!(last_event(), Some(Event::kitties(crate::Event::<Test>::KittyCreated(100, 0, kitty))));
    });
}

#[test]
fn can_breed_test() {
    new_test_ext().execute_with(|| {
        assert_ok!(KittiesModule::create(Origin::signed(100)));
        assert_eq!(KittiesModule::next_kitty_id(), 1);

		// bumps extrinsic index for the next DNA generation
        System::set_extrinsic_index(1);

        assert_ok!(KittiesModule::create(Origin::signed(100)));
        assert_eq!(KittiesModule::next_kitty_id(), 2);

        assert_noop!(KittiesModule::breed(Origin::signed(100), 0, 11), Error::<Test>::KittyNotOwned);
        assert_noop!(KittiesModule::breed(Origin::signed(100), 0, 0),  Error::<Test>::KittiesBredFromSameGenderCouple);
        assert_noop!(KittiesModule::breed(Origin::signed(101), 0, 1),  Error::<Test>::KittyNotOwned);

        assert_ok!(KittiesModule::breed(Origin::signed(100), 0, 1));
        assert_eq!(KittiesModule::next_kitty_id(), 3);

        let kitty = Kitty([59, 254, 219, 122, 245, 239, 191, 125, 255, 239, 247, 247, 251, 239, 247, 254]);

        assert_eq!(KittiesModule::kitties(100, 2), Some(kitty.clone()));

		let momma = KittiesModule::kitties(100, 0).unwrap();
		assert_eq!(momma.get_gender(), Gender::Female);
		let poppa = KittiesModule::kitties(100, 1).unwrap();
		assert_eq!(poppa.get_gender(), Gender::Male);
        assert_eq!(last_event(), Some(Event::kitties(crate::Event::<Test>::KittyBred(100u64, 2u32, kitty, momma, poppa))));
    });
}

#[test]
fn transfer_test() {
    new_test_ext().execute_with(|| {
		let me_id = 100;
		let me = Origin::signed(me_id);
		let another_id = 101;
        assert_ok!(KittiesModule::create(me.clone()));
		let kitty1 = KittiesModule::kitties(me_id, 0).unwrap();

		// invalid transfer of kitty not owned by myself
		assert_noop!(KittiesModule::transfer(me.clone(), me_id, 10), Error::<Test>::KittyNotOwned);

		// valid transfer to myself, expect no event
		assert_ok!(KittiesModule::transfer(me.clone(), me_id, 0));
		let kitty2 = KittiesModule::kitties(me_id, 0).unwrap();
		assert_eq!(kitty1, kitty2);

		// valid transfer to another, expect move of kitty and event
		assert_ok!(KittiesModule::transfer(me.clone(), another_id, 0));
		assert_eq!(KittiesModule::kitties(me_id, 0), None);
		let kitty3 = KittiesModule::kitties(another_id, 0).unwrap();
		assert_eq!(kitty1, kitty3);
		assert_eq!(last_event(), Some(Event::kitties(crate::Event::<Test>::KittyTransfered(me_id, another_id, 0, kitty3))));
	});
}

#[test]
fn set_price_test() {
    new_test_ext().execute_with(|| {
		let me_id = 100;
		let me = Origin::signed(me_id);
		let another_id = 101;
		// Kitties::<Test>::insert(me_id, 0, Kitty([1,2,3,4,5,6,7,8,9,0,1,2,3,4,5,6])); // enter raw storage - note - other creation items (next_id) aren't updated...
		// or preferred:
		assert_ok!(KittiesModule::create(me.clone()));
		assert_ok!(KittiesModule::set_price(me.clone(), 0, None));
		assert_eq!(last_event(), Some(Event::kitties(RawEvent::KittyPriceSet(100, 0, None))));  // set it to None (was None, but still, want to send notification of success)
		assert_eq!(Prices::<Test>::get(0), None);

		assert_ok!(KittiesModule::set_price(me.clone(), 0, Some(100_u64)));
		assert_eq!(last_event(), Some(Event::kitties(RawEvent::KittyPriceSet(100, 0, Some(100_u64)))));
		assert_eq!(Prices::<Test>::get(0), Some(100_u64));

		// set price on someone else's kitty
		assert_noop!(KittiesModule::set_price(me.clone(), another_id, None), Error::<Test>::KittyNotOwned);
		assert_eq!(Prices::<Test>::get(0), Some(100_u64));
	});
}

#[test]
fn set_buy() {
    new_test_ext().execute_with(|| {
		let me_id = 100;
		let me = Origin::signed(me_id);
		let poor_buyer = 200;
		let rich_buyer = 300;

		assert_ok!(KittiesModule::create(me.clone()));

		// try to buy unpriced kitty
		assert_noop!(KittiesModule::buy(me.clone(), poor_buyer, 0, 250), Error::<Test>::KittyNotForSale);

		// try to buy non existant kitty
		assert_noop!(KittiesModule::buy(me.clone(), poor_buyer, 10, 250), Error::<Test>::KittyNotOwned);

		// try to buy below price
		assert_ok!(KittiesModule::set_price(me.clone(), 0, Some(200)));
		assert_noop!(KittiesModule::buy(me.clone(), poor_buyer, 0, 10), Error::<Test>::KittyPriceTooLow);

		// fail to buy due to depleting balance to 0
		assert_ok!(KittiesModule::set_price(me.clone(), 0, Some(200)));
		assert_noop!(KittiesModule::buy(me.clone(), poor_buyer, 0, 1000), pallet_balances::Error::<Test, _>::KeepAlive);

		// fail to buy due depleting the balance < 0
		assert_ok!(KittiesModule::set_price(me.clone(), 0, Some(250)));
		assert_eq!(Prices::<Test>::get(0), Some(250));
		assert_noop!(KittiesModule::buy(me.clone(), poor_buyer, 0, 1000), pallet_balances::Error::<Test, _>::InsufficientBalance);

		// buy ok! and not be able to buy again due to kitty being unpriced post transfer
		assert_ok!(KittiesModule::buy(me.clone(), rich_buyer, 0, 1000));
		assert_eq!(last_event(), Some(Event::kitties(RawEvent::KittyBought(100, rich_buyer, 0, 250))));
		assert_eq!(Prices::<Test>::get(0), None);
		assert!(! Kitties::<Test>::contains_key(me_id, 0));
		assert!(! Kitties::<Test>::contains_key(poor_buyer, 0));
		assert!(Kitties::<Test>::contains_key(rich_buyer, 0));
		assert_eq!(Balances::free_balance(me_id), 350);
		assert_eq!(Balances::free_balance(poor_buyer), 200);
		assert_eq!(Balances::free_balance(rich_buyer), 50);
	});
}

#[test]
fn gender() {
    assert_eq!(Kitty([0; 16]).get_gender(), Gender::Male);
    assert_eq!(Kitty([1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]).get_gender(), Gender::Female);
}

#[test]
fn mix_dna_test() {
	let dna1: [u8; 16] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
	let dna2: [u8; 16] = [101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111, 112, 113, 114, 115, 116];
	assert_eq!(dna1, mix_dna([0u8; 16], dna1, dna2));
	assert_eq!(dna2, mix_dna([255u8; 16], dna1, dna2));
	assert_eq!(
		[1, 102, 3, 104, 5, 106, 7, 108, 9, 110, 11, 112, 13, 114, 15, 116],
		mix_dna([0u8, 255u8, 0u8, 255u8, 0u8, 255u8, 0u8, 255u8, 0u8, 255u8, 0u8, 255u8, 0u8, 255u8, 0u8, 255u8], dna1, dna2));
}

#[test]
fn mix_get_female_male_test() {
	let male = Kitty([2u8; 16]);
	let female = Kitty([1u8; 16]);
	assert_eq!(Some((&female, &male)), get_female_male(&male, &female));
	assert_eq!(Some((&female, &male)), get_female_male(&female, &male));
	assert_eq!(None, get_female_male(&female, &female));
	assert_eq!(None, get_female_male(&male, &male));
}


#[test]
fn combine_dna_works() {
	assert_eq!(mix_dna([0b00001111; 16], [0b11111111; 16], [0b00000000; 16]), [0b11110000; 16]);
	assert_eq!(mix_dna([0b11001100; 16], [0b10101010; 16], [0b11110000; 16]), [0b11100010; 16]);
}