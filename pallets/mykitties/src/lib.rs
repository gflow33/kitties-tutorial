#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use frame_system::pallet_prelude::*;
	use frame_support::{
		dispatch::{DispatchResult, DispatchResultWithPostInfo},
		pallet_prelude::*,
		sp_runtime::traits::{Hash, Zero},
		traits::{Currency, ExistenceRequirement, Randomness},
		transactional,
	};
	use sp_io::hashing::blake2_128;

	#[cfg(feature = "std")]
	use serde::{Deserialize, Serialize};

	type AccountOf<T> = <T as frame_system::Config>::AccountId;
	type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
    
    // TODO Part II: Struct for holding Kitty information.

	#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	#[scale_info(skip_type_params(T))]
	#[codec(mel_bound())]
	pub struct Kitty<T: Config> {
		pub dna: [u8; 16],
		pub price: Option<BalanceOf<T>>,
		pub gender: Gender,
		pub owner: AccountOf<T>,
	}

    // TODO Part II: Enum and implementation to handle Gender type in Kitty struct.

	#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	pub enum Gender {
		Male,
		Female,
	}

	impl Default for Gender {
		fn default() -> Self {
			Gender::Male
		}
	}

	
	#[pallet::pallet]
	#[pallet::generate_store(trait Store)]
	pub struct Pallet<T>(_);
// 
	/// Configure the pallet by specifying the parameters and types it depends on.
	#[pallet::config]
	pub trait Config: pallet_balances::Config + frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		// The Currency handler for the Kitties pallet.
		type Currency: Currency<Self::AccountId>;

		// TODO Part II: Specify the custom types for our runtime.
		type KittyRandomness: Randomness<Self::Hash, Self::BlockNumber>;

		#[pallet::constant]
		type MaxKittiesOwned: Get<u32>;
	}

	// Errors.
	#[pallet::error]
	pub enum Error<T> {
		// TODO Part III
        
		/// Handles arithemtic overflow when incrementing the Kitty counter.
        KittyCntOverflow,
        /// An account cannot own more Kitties than MaxKittyCount.
        ExceedMaxKittyOwned,
        /// Buyer cannot be the owner.
        BuyerIsKittyOwner,
        /// Cannot transfer a kitty to its owner.
        TransferToSelf,
        /// Handles checking whether the Kitty exists.
        KittyNotExist,
        /// Handles checking that the Kitty is owned by the account transferring, buying or setting a price for it.
        NotKittyOwner,
        /// Ensures the Kitty is for sale.
        KittyNotForSale,
        /// Ensures that the buying price is greater than the asking price.
        KittyBidPriceTooLow,
        /// Ensures that an account has enough funds to purchase a Kitty. 
        NotEnoughBalance,
	}

	#[pallet::event]
	// #[pallet::metadata(T::AccountId = "AccountId")]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		// TODO Part III
        /// A new Kitty was sucessfully created. \[sender, kitty_id\]
		Created(T::AccountId, T::Hash),
        /// Kitty price was sucessfully set. \[sender, kitty_id, new_price\]
		PriceSet(T::AccountId, T::Hash, Option<BalanceOf<T>>),
        /// A Kitty was sucessfully transferred. \[from, to, kitty_id\]
		Transfered(T::AccountId, T::AccountId, T::Hash),
        /// A Kitty was sucessfully bought. \[buyer, seller, kitty_id, bid_price\]
		Bought(T::AccountId, T::AccountId, T::Hash, BalanceOf<T>),
	}

	#[pallet::storage]
	#[pallet::getter(fn all_kitties_count)]
	pub(super) type AllKittiesCount<T: Config> = StorageValue<_, u64, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn kitties)]
	/// Stores a Kitty's unique traits, owner and price.
	pub(super) type Kitties<T: Config> = 
    StorageMap<
        _, 
        Twox64Concat, 
        T::Hash, 
        Kitty<T>
        >;

	#[pallet::storage]
	#[pallet::getter(fn kitties_owned)]
	pub(super) type KittiesOwned<T: Config> = StorageMap<
		_,
		Twox64Concat,
		T::AccountId,
		BoundedVec<T::Hash, T::MaxKittiesOwned>,
		ValueQuery,
	>;

	// TODO Part II: Remaining storage items.

	// TODO Part III: Our pallet's genesis configuration.
/*
	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
	pub kitties: Vec<(T::AccountId, [u8; 16], Gender)>,
	}

	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
	fn default() -> GenesisConfig<T> {
	 	GenesisConfig { kitties: vec![] }
	    }
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
	fn build(&self) {
	    for (acc, dna, gender) in &self.kitties {
			let _ = <Pallet<T>>::mint(acc, Some(gender.clone()), Some(dna.clone()));
		    }
 	   }
	}
*/
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		// TODO Part III: create_kitty

		#[pallet::weight(100)]
		pub fn create_kitty(origin: OriginFor<T>) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			let kitty_id = Self::mint(&sender, None, None);
            //Self::deposit_event(Event::Created(sender, kitty_id));
			Ok(())
		}
        
        // TODO Part III: set_price

		#[pallet::weight(100)]
		pub fn set_price(
			origin: OriginFor<T>,
			kitty_id: T::Hash,
			new_price: Option<BalanceOf<T>>,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			ensure!(Self::is_kitty_owner(&kitty_id, &sender)?, <Error<T>>::NotKittyOwner);

			let mut kitty = Self::kitties(&kitty_id).ok_or(<Error<T>>::KittyNotExist)?;
			kitty.price = new_price;

			<Kitties<T>>::insert(&kitty_id, kitty);

			Self::deposit_event(Event::PriceSet(sender, kitty_id, new_price));

			Ok(())
		}

        // TODO Part III: transfer_kitty

		#[pallet::weight(100)]
		pub fn transfer_kitty(
			origin: OriginFor<T>,
			kitty_id: T::Hash,
			to: T::AccountId,
		) -> DispatchResult {
			let from = ensure_signed(origin)?;
			ensure!(Self::is_kitty_owner(&kitty_id, &from)?, <Error<T>>::NotKittyOwner);
			ensure!(from != to, <Error<T>>::TransferToSelf);
			let to_owned = <KittiesOwned<T>>::get(&to);

			ensure!(
				(to_owned.len() as u32) < T::MaxKittiesOwned::get(),
				<Error<T>>::ExceedMaxKittyOwned
                //MaxKittiesOwned
			);
			Self::transfer_kitty_to(&kitty_id, &to)?;

			Self::deposit_event(Event::Transfered(from, to, kitty_id));

			Ok(())
		}

        // TODO Part III: buy_kitty

		#[pallet::weight(100)]
		pub fn buy_kitty(
			origin: OriginFor<T>,
			kitty_id: T::Hash,
			price: BalanceOf<T>,
		) -> DispatchResult {
			let buyer = ensure_signed(origin)?;

			let kitty = Self::kitties(&kitty_id).ok_or(<Error<T>>::KittyNotExist)?;

			ensure!(kitty.owner != buyer, <Error<T>>::BuyerIsKittyOwner);

			if let Some(ask_price) = kitty.price {
				ensure!(ask_price <= price, <Error<T>>::KittyBidPriceTooLow);
			} else {
				Err(<Error<T>>::KittyNotForSale)?;
			}

			ensure!(T::Currency::free_balance(&buyer) >= price, <Error<T>>::NotEnoughBalance);
			let to_owned = <KittiesOwned<T>>::get(&buyer);
			ensure!(
				(to_owned.len() as u32) < T::MaxKittiesOwned::get(),
				<Error<T>>::ExceedMaxKittyOwned
			);

			let seller = kitty.owner.clone();

			T::Currency::transfer(&buyer, &seller, price, ExistenceRequirement::KeepAlive)?;
			Self::transfer_kitty_to(&kitty_id, &buyer)?;

			Self::deposit_event(Event::Bought(buyer, seller, kitty_id, price));

			Ok(())
		}

        // TODO Part III: breed_kitty

		#[pallet::weight(100)]
		pub fn breed_kitty(origin: OriginFor<T>, kid1: T::Hash, kid2: T::Hash) -> DispatchResult {
			let breeder = ensure_signed(origin)?;
			ensure!(Self::is_kitty_owner(&kid1, &breeder)?, <Error<T>>::NotKittyOwner);
			ensure!(Self::is_kitty_owner(&kid2, &breeder)?, <Error<T>>::NotKittyOwner);

			let new_dna = Self::breed_dna(&kid1, &kid2)?;
			Self::mint(&breeder, None, Some(new_dna))?;

			Ok(())
		}
	
	}

	impl<T: Config> Kitty<T> {
		pub fn gender(dna: T::Hash) -> Gender {
			if dna.as_ref()[0] % 2 == 0 {
				Gender::Male
			} else {
				Gender::Female
			}
		}
	}

	impl<T: Config> Pallet<T> {
		// TODO Part III: helper functions for dispatchable functions

		// TODO: increment_nonce, random_hash, mint, transfer_from

		fn gen_gender() -> Gender {
			let random = T::KittyRandomness::random(&b"gender"[..]).0;
			match random.as_ref()[0] % 2 {
				0 => Gender::Male,
				_ => Gender::Female,
			}
		}

		fn mint(
			owner: &T::AccountId,
			gender: Option<Gender>,
			dna: Option<[u8; 16]>,
		) -> Result<T::Hash, Error<T>> {
			let kitty = Kitty::<T> {
				dna: dna.unwrap_or_else(Self::gen_dna),
				price: None,
				gender: gender.unwrap_or_else(Self::gen_gender),
				owner: owner.clone(),
			};
			let kitty_id = T::Hashing::hash_of(&kitty);
			let new_cnt =
				Self::all_kitties_count().checked_add(1).ok_or(<Error<T>>::KittyCntOverflow)?;

			<KittiesOwned<T>>::try_mutate(&owner, |kitty_vec| kitty_vec.try_push(kitty_id))
				.map_err(|_| <Error<T>>::ExceedMaxKittyOwned)?;

			<Kitties<T>>::insert(kitty_id, kitty);
			<AllKittiesCount<T>>::put(new_cnt);

			Self::deposit_event(Event::Created(owner.clone(), kitty_id));

			Ok(kitty_id)
		}

		fn gen_dna() -> [u8; 16] {
			let payload = (
				T::KittyRandomness::random(&b"gender"[..]).0,
				<frame_system::Pallet<T>>::block_number(),
			);

			payload.using_encoded(blake2_128)
		}

		pub fn breed_dna(kid1: &T::Hash, kid2: &T::Hash) -> Result<[u8; 16], Error<T>> {
			let dna1 = Self::kitties(kid1).ok_or(<Error<T>>::KittyNotExist)?.dna;
			let dna2 = Self::kitties(kid2).ok_or(<Error<T>>::KittyNotExist)?.dna;

			let mut new_dna = Self::gen_dna();
			for i in 0..new_dna.len() {
				new_dna[i] = (new_dna[i] & dna1[i]) | (!new_dna[i] & dna2[i]);
			}

			Ok(new_dna)
		}

		pub fn is_kitty_owner(kitty_id: &T::Hash, owner: &T::AccountId) -> Result<bool, Error<T>> {
			match Self::kitties(kitty_id) {
				Some(kitty) => Ok(kitty.owner == *owner),
				None => Err(<Error<T>>::KittyNotExist),
			}
		}

		// #[transactional]
		pub fn transfer_kitty_to(kitty_id: &T::Hash, to: &T::AccountId) -> Result<(), Error<T>> {
			let mut kitty = Self::kitties(&kitty_id).ok_or(<Error<T>>::KittyNotExist)?;
			let prev_owner = kitty.owner.clone();

			<KittiesOwned<T>>::try_mutate(&prev_owner, |owned| {
				if let Some(ind) = owned.iter().position(|&id| id == *kitty_id) {
					owned.swap_remove(ind);
					return Ok(())
				}
				Err(())
			})
			.map_err(|_| <Error<T>>::KittyNotExist)?;
			let kitty_owner = to.clone();
			kitty.price = None;

			<Kitties<T>>::insert(kitty_id, kitty);
			<KittiesOwned<T>>::try_mutate(to, |vec| vec.try_push(*kitty_id))
				.map_err(|_| <Error<T>>::ExceedMaxKittyOwned)?;

			Ok(())
		}
	}
}