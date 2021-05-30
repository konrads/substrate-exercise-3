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
	type AccountData = ();
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = SS58Prefix;
}

impl Config for Test {
	type Event = Event;
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut t: sp_io::TestExternalities = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap().into();
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
		assert!(KittiesModule::kitties(me_id, 0).is_none());
		let kitty3 = KittiesModule::kitties(another_id, 0).unwrap();
		assert_eq!(kitty1, kitty3);
		assert_eq!(last_event(), Some(Event::kitties(crate::Event::<Test>::KittyTransfered(me_id, another_id, 0, kitty3))));
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