#![cfg_attr(not(feature = "std"), no_std)]

// mod kitties;
//
use codec::{Encode, Decode};
use frame_support::{
	decl_module, decl_storage, decl_event, decl_error, ensure, StorageValue, StorageDoubleMap,
	traits::Randomness, RuntimeDebug,
};
use frame_support::dispatch::DispatchResult;
use sp_io::hashing::blake2_128;
use frame_system::ensure_signed;

// #[derive(Encode, Decode, Clone, Copy, RuntimeDebug, PartialEq, Eq)]
#[derive(PartialEq, Eq)]
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

pub trait Config: frame_system::Config {
	type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;
}

decl_storage! {
	trait Store for Module<T: Config> as Kitties {
		/// Stores all the kitties, key is the kitty id
		pub Kitties get(fn kitties): double_map hasher(blake2_128_concat) T::AccountId, hasher(blake2_128_concat) u32 => Option<Kitty>;
		/// Stores parents
		pub Parents get(fn parents): double_map hasher(blake2_128_concat) T::AccountId, hasher(blake2_128_concat) u32 => Option<(u32, u32)>;
		/// Stores the next kitty ID
		pub NextKittyId get(fn next_kitty_id): u32;
	}
}

decl_event! {
	pub enum Event<T> where
		<T as frame_system::Config>::AccountId,
	{
		/// A kitty is created. \[owner, kitty_id, kitty\]
		KittyCreated(AccountId, u32, Kitty),

		/// A kitty is bred. \[owner, kitty_child, momma_kitty, papa_kitty\]
		KittyBred(AccountId, u32, Kitty, Kitty, Kitty),
	}
}

decl_error! {
	pub enum Error for Module<T: Config> {
		KittiesIdOverflow,
		KittyNotOwned, // get error if use (u32) param: ^ no rules expected this token in macro call
		KittiesBredFromSameGenderCouple,
	}
}

decl_module! {
	pub struct Module<T: Config> for enum Call where origin: T::Origin {
		type Error = Error<T>;

		fn deposit_event() = default;

		/// Create a new kitty
		#[weight = 1000]
		pub fn create(origin) {
			let sender = ensure_signed(origin)?;

			// ensure no id overflow
			// FIXME: ks - try_mutate - how would i know that?
			NextKittyId::try_mutate(|curr_id_ref| -> DispatchResult {
				let curr_id = *curr_id_ref;

				// Generate a random 128bit value
				// FIXME: ks - using_encoded() on a tuple?
				let payload = (
					<pallet_randomness_collective_flip::Module<T> as Randomness<T::Hash>>::random_seed(),
					&sender,
					<frame_system::Module<T>>::extrinsic_index(),
				);
				let dna = payload.using_encoded(blake2_128);

				// Create and store kitty
				let kitty = Kitty(dna);
				// FIXME: ks - is also available on Self::kitties? how?
				Kitties::<T>::insert(&sender, curr_id, kitty.clone());

				let next_kitty_id = curr_id.checked_add(1).ok_or(Error::<T>::KittiesIdOverflow)?;
				*curr_id_ref = next_kitty_id;
				NextKittyId::put(next_kitty_id);
				// Emit event
				Self::deposit_event(RawEvent::KittyCreated(sender, next_kitty_id, kitty));

				Ok(())
			})?
		}

		/// Design breed feature for kitties pallet
        /// Requirements
        /// a. A kitty have gender, based on DNA
        /// b. Kitty owner can choose two kitties with opposite gender to breed a new kitten
        /// c. New kitten should inherits the DNA from parents
		#[weight = 1000]
		pub fn breed(origin, parent1_id: u32, parent2_id: u32) {
			let sender = ensure_signed(origin)?;
			let parent1 = Self::kitties(&sender, parent1_id).ok_or(Error::<T>::KittyNotOwned)?;
			let parent2 = Self::kitties(&sender, parent2_id).ok_or(Error::<T>::KittyNotOwned)?;
			let (momma, pappa) = get_female_male(&parent1, &parent2).ok_or(Error::<T>::KittiesBredFromSameGenderCouple)?;

			// obtain child dna from parents' dnas
			let mixer: [u8; 16] = <pallet_randomness_collective_flip::Module<T> as Randomness<T::Hash>>::random_seed().using_encoded(blake2_128);
			let child_dna: [u8; 16] = mix_dna(mixer, momma.0, pappa.0);
			let (momma_id, poppa_id) = if parent1.get_gender() == Gender::Female {
				(parent1_id, parent2_id)
			} else {
				(parent2_id, parent1_id)
			};

			NextKittyId::try_mutate(|curr_id_ref| -> DispatchResult {
				let curr_id = *curr_id_ref;
				let child_id = curr_id.checked_add(1).ok_or(Error::<T>::KittiesIdOverflow)?;
				let child = Kitty(child_dna);

				Kitties::<T>::insert(&sender, curr_id, child.clone());
				Parents::<T>::insert(&sender, curr_id, (momma_id, poppa_id));
				NextKittyId::put(child_id);
				Self::deposit_event(RawEvent::KittyBred(sender, child_id, child, (*momma).clone(), (*pappa).clone()));

				Ok(())
			})?
		}

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
mod tests {
    use super::*;

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
    fn mix_get_femae_male_test() {
		let male = Kitty([2u8; 16]);
		let female = Kitty([1u8; 16]);
		assert_eq!(Some((&female, &male)), get_female_male(&male, &female));
		assert_eq!(Some((&female, &male)), get_female_male(&female, &male));
		assert_eq!(None, get_female_male(&female, &female));
		assert_eq!(None, get_female_male(&male, &male));
    }
}
