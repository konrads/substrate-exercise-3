#![cfg_attr(not(feature = "std"), no_std)]

// mod kitties;
//
use std::fmt;
use codec::{Encode, Decode};
use frame_support::{
	decl_module, decl_storage, decl_event, decl_error, StorageValue, StorageDoubleMap, Parameter,
	traits::{Randomness, Currency, ExistenceRequirement}, ensure,
	RuntimeDebug,
};
use frame_support::dispatch::{DispatchError, DispatchResult};
use sp_io::hashing::blake2_128;
use frame_system::ensure_signed;
use sp_runtime::traits::{AtLeast32BitUnsigned, Bounded, One, CheckedAdd};

// #[derive(Encode, Decode, Clone, Copy, RuntimeDebug, PartialEq, Eq)]
#[derive(PartialEq, Eq, RuntimeDebug)]
enum Gender {
    Male,
    Female
}

#[derive(Encode, Decode, Clone, RuntimeDebug, PartialEq, Eq)]
pub struct Kitty(pub [u8; 16]);

impl Kitty {
    fn get_gender(&self) -> Gender {
        match self.0[0] % 2 {
            0 => Gender::Male,
            _ => Gender::Female
        }
    }
}

pub trait Config: pallet_balances::Config {
	type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;
	type Currency: Currency<Self::AccountId>;
	// type Randomness: Randomness<Self::Hash>;
	type KittyIndex: Parameter + AtLeast32BitUnsigned + Bounded + Default + Copy + fmt::Display;
}

//type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
type BalanceOf<T> = <T as pallet_balances::Config>::Balance;

decl_storage! {
	trait Store for Module<T: Config> as Kitties {
		/// Stores all the kitties, key is the kitty id
		/// Implemented via https://substrate.dev/rustdocs/v3.0.0/frame_support/storage/trait.StorageDoubleMap.html
		pub Kitties get(fn kitties): double_map hasher(blake2_128_concat) T::AccountId, hasher(blake2_128_concat) T::KittyIndex => Option<Kitty>;
		/// Stores parent ids, key is the child kitty id
		/// Implemented via https://substrate.dev/rustdocs/v3.0.0/frame_support/storage/trait.StorageMap.html
		pub Parents get(fn parents): map hasher(blake2_128_concat) T::KittyIndex => Option<(T::KittyIndex, T::KittyIndex)>;
		pub Prices get(fn prices): map hasher(blake2_128_concat) T::KittyIndex => Option<BalanceOf<T>>;
		/// Stores the next kitty ID
		// Implemented via https://substrate.dev/rustdocs/v3.0.0/frame_support/storage/trait.StorageValue.html
		pub NextKittyId get(fn next_kitty_id): T::KittyIndex;
	}
}

decl_event! {
	pub enum Event<T> where
		<T as frame_system::Config>::AccountId,
		Balance = BalanceOf<T>,
		<T as Config>::KittyIndex,
	{
		/// A kitty is created. \[owner, kitty_id, kitty\]
		KittyCreated(AccountId, KittyIndex, Kitty),

		/// A kitty is bred. \[owner, kitty_id, kitty_child, momma_kitty, papa_kitty\]
		KittyBred(AccountId, KittyIndex, Kitty, Kitty, Kitty),

		/// A kitty is transfered. \[owner, new_owner, kitty_id, kitty\]
		KittyTransfered(AccountId, AccountId, KittyIndex, Kitty),

		/// A kitty price is set. \[owner, kitty_id, price\]
		KittyPriceSet(AccountId, KittyIndex, Option<Balance>),

		/// A kitty is bought. \[owner, new_owner, kitty_id, price\]
		KittyBought(AccountId, AccountId, KittyIndex, Balance),
	}
}

decl_error! {
	pub enum Error for Module<T: Config> {
		KittiesIdOverflow,
		KittyNotOwned, // Note: decl_error! doesn't allow for parametrized Enum values, ie. will get error if use a param er (u32): ^ no rules expected this token in macro call
		KittiesBredFromSameGenderCouple,
		KittyNotForSale,
		KittyPriceTooLow,
	}
}

decl_module! {
	pub struct Module<T: Config> for enum Call where origin: T::Origin {
		type Error = Error<T>;

		fn deposit_event() = default;

		/// Create a new kitty
		#[weight = 1000]
		pub fn create(origin) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			// ensure no id overflow
			let kitty_id = Self::get_next_kitty_id()?;

			// FIXME: discover how using_encoded() works on such tuple...
			// Generate a random 128bit value
			let dna = Self::random_value(&sender);

			// Create and store kitty
			let kitty = Kitty(dna);
			// note, setter isn't created as part of doublemap decl_storage!
			Kitties::<T>::insert(&sender, kitty_id, kitty.clone());

			// Emit event
			Self::deposit_event(RawEvent::KittyCreated(sender, kitty_id, kitty));

			frame_support::debug::RuntimeLogger::init();
			frame_support::debug::info!("##### create(): dna: {:?}, next_kitty_id: {}", dna, kitty_id);

			Ok(())
		}

		/// Design breed feature for kitties pallet
        /// Requirements
        /// a. A kitty have gender, based on DNA
        /// b. Kitty owner can choose two kitties with opposite gender to breed a new kitten
        /// c. New kitten should inherits the DNA from parents
		#[weight = 1000]
		pub fn breed(origin, parent1_id: T::KittyIndex, parent2_id: T::KittyIndex) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			let parent1 = Self::kitties(&sender, parent1_id).ok_or(Error::<T>::KittyNotOwned)?;
			let parent2 = Self::kitties(&sender, parent2_id).ok_or(Error::<T>::KittyNotOwned)?;
			let (momma, pappa) = get_female_male(&parent1, &parent2).ok_or(Error::<T>::KittiesBredFromSameGenderCouple)?;

			let child_id = Self::get_next_kitty_id()?;

			// obtain child dna from parents' dnas
			let mixer: [u8; 16] = Self::random_value(&sender);
			let child_dna: [u8; 16] = mix_dna(mixer, momma.0, pappa.0);
			// ensure recording tuple order: momma, pappa
			let (momma_id, poppa_id) = if parent1.get_gender() == Gender::Female {
				(parent1_id, parent2_id)
			} else {
				(parent2_id, parent1_id)
			};

			let child = Kitty(child_dna);
			Kitties::<T>::insert(&sender, child_id, child.clone());
			Parents::<T>::insert(child_id, (momma_id, poppa_id));

			frame_support::debug::RuntimeLogger::init();
			frame_support::debug::info!("##### breed(): child dna: {:?}, momma_id: {}, poppa_id: {}", child_dna, momma_id, poppa_id);

			Self::deposit_event(RawEvent::KittyBred(sender, child_id, child, (*momma).clone(), (*pappa).clone()));

			Ok(())
		}

		/// Design transfer feature
		/// a. kitty owner should be able to transfer kitty to someone else
		#[weight = 1000]
		pub fn transfer(origin, new_owner: T::AccountId, kitty_id: T::KittyIndex) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			Kitties::<T>::try_mutate_exists(sender.clone(), kitty_id, |kitty| -> DispatchResult {
				if sender == new_owner && kitty.is_some() {
					Ok(())					
				} else {
					match kitty.take() {  // not sure why, but take() is required to remove from storage
						None    => Err(Error::<T>::KittyNotOwned.into()),
						Some(k) => {
							Kitties::<T>::insert(&new_owner, kitty_id, k.clone());
							// Prices::<T>::insert(kitty_id, None);
							Self::deposit_event(RawEvent::KittyTransfered(sender, new_owner, kitty_id, k));
							Ok(())
						}
					}
				}
			})
		}

		#[weight = 1000]
		pub fn set_price(origin, kitty_id: T::KittyIndex, new_price: Option<BalanceOf<T>>) -> DispatchResult {
			// bryan's impl
			let sender = ensure_signed(origin)?;
			ensure!(Kitties::<T>::contains_key(&sender, kitty_id), Error::<T>::KittyNotOwned);  // more performant than fetch cause no serialization
			Prices::<T>::mutate_exists(kitty_id, |price| -> () {
				*price = new_price;
			});  // if returning a None, force erasue
			Self::deposit_event(RawEvent::KittyPriceSet(sender, kitty_id, new_price));
			Ok(())

			// my impl - equivalent to above
			// let sender = ensure_signed(origin)?;
			// Self::kitties(&sender, kitty_id).ok_or(Error::<T>::KittyNotOwned)?;
			// match new_price {
			// 	Some(p) => { Prices::<T>::insert(kitty_id, p); }
			// 	None => ()
			// }
			// Self::deposit_event(RawEvent::KittyPriceSet(sender, kitty_id, new_price));
			// Ok(())
		}

		#[weight = 1000]
		pub fn buy(origin, new_owner: T::AccountId, kitty_id: T::KittyIndex, max_bid: BalanceOf<T>) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			// transfer if prices are below max bid
			Kitties::<T>::try_mutate_exists(sender.clone(), kitty_id, |kitty| -> DispatchResult {
				let kitty = kitty.take().ok_or(Error::<T>::KittyNotOwned)?;   // will remove from map!
				Prices::<T>::try_mutate_exists(kitty_id, |price| -> DispatchResult {
					let price = price.take().ok_or(Error::<T>::KittyNotForSale)?;  // will remove from map!
					ensure!(price <= max_bid, Error::<T>::KittyPriceTooLow);
					<pallet_balances::Module<T> as Currency<T::AccountId>>::transfer(&new_owner, &sender, price, ExistenceRequirement::KeepAlive)?;  // KeepAlive = ensure enough funds in account to keep account alive
					Kitties::<T>::insert(&new_owner, kitty_id, kitty);
					Self::deposit_event(RawEvent::KittyBought(sender, new_owner, kitty_id, price));
					Ok(())
				})
			})
		}

	}
}

// from Bryan's answers
impl<T: Config> Module<T> {
	fn get_next_kitty_id() -> sp_std::result::Result<T::KittyIndex, DispatchError> {
		NextKittyId::<T>::try_mutate(|next_id| -> sp_std::result::Result<T::KittyIndex, DispatchError> {
			let current_id = *next_id;
			*next_id = next_id.checked_add(&One::one()).ok_or(Error::<T>::KittiesIdOverflow)?;
			Ok(current_id)
		})
	}

	fn random_value(sender: &T::AccountId) -> [u8; 16] {
		let payload = (
			<pallet_randomness_collective_flip::Module<T> as Randomness<T::Hash>>::random_seed(),
			&sender,
			<frame_system::Module<T>>::extrinsic_index(),
		);
		payload.using_encoded(blake2_128)
	}
}

fn get_female_male<'a>(kitty1: &'a Kitty, kitty2: &'a Kitty) -> Option<(&'a Kitty, &'a Kitty)> {
	match (kitty1.get_gender(), kitty2.get_gender()) {
		(Gender::Female, Gender::Male) => Some((kitty1, kitty2)),
		(Gender::Male, Gender::Female) => Some((kitty2, kitty1)),
		_ => None
	}
}

fn mix_dna(mixer: [u8; 16], dna1: [u8; 16], dna2: [u8; 16]) -> [u8; 16] {
    let mut res: [u8; 16] = [0u8; 16];
    for i in 0..mixer.len() {
        res[i] = (!mixer[i] & dna1[i]) + (mixer[i] & dna2[i]);
    }
    res
}

#[cfg(test)]
mod tests;
