#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

/// The log target of this pallet.
pub const LOG_TARGET: &str = "runtime::nft_computing";

// Syntactic sugar for logging.
#[macro_export]
macro_rules! log {
	($level:tt, $patter:expr $(, $values:expr)* $(,)?) => {
		log::$level!(
			target: $crate::LOG_TARGET,
			concat!("[{:?}] ", $patter), <frame_system::Pallet<T>>::block_number() $(, $values)*
		)
	};
}

use frame_support::{
	traits::{
		tokens::nonfungibles_v2::{self, *},
		Currency, ReservableCurrency,
	},
};
use pallet_nfts::{
	MintType, CollectionSettings, CollectionSetting, ItemSettings, ItemSetting,
	ItemConfig
};

pub(crate) type BalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

pub(crate) type CollectionIdOf<T> =
	<<T as Config>::NFTCollection as nonfungibles_v2::Inspect<<T as frame_system::Config>::AccountId>>::CollectionId;

pub(crate) type CollectionConfigOf<T> = pallet_nfts::CollectionConfig<
	BalanceOf<T>,
	<T as frame_system::Config>::BlockNumber,
	CollectionIdOf<T>
>;
pub(crate) type MintSettingsOf<T> = pallet_nfts::MintSettings<
	BalanceOf<T>,
	<T as frame_system::Config>::BlockNumber,
	CollectionIdOf<T>
>;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	/// The current storage version.
	const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::storage_version(STORAGE_VERSION)]
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// The system's currency for payment.
		type Currency: ReservableCurrency<Self::AccountId>;

		type NFTCollection: nonfungibles_v2::Create<Self::AccountId, CollectionConfigOf<Self>> +
							nonfungibles_v2::Destroy<Self::AccountId> +
							nonfungibles_v2::Mutate<Self::AccountId, ItemConfig>;
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		CollectionCreated { who: T::AccountId, collection_id: CollectionIdOf<T> },
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		NotTheOwner,
		WorkerNotExists,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(0)]
		pub fn create_collection(
			origin: OriginFor<T>
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let collection_config = CollectionConfigOf::<T> {
				settings: CollectionSettings::from_disabled(
					CollectionSetting::TransferableItems |
						CollectionSetting::UnlockedMetadata |
						CollectionSetting::UnlockedAttributes |
						CollectionSetting::UnlockedMaxSupply
				),
				max_supply: None,
				mint_settings: MintSettingsOf::<T> {
					mint_type: MintType::Public,
					price: None,
					start_block: None,
					end_block: None,
					default_item_settings: ItemSettings::from_disabled(
						ItemSetting::Transferable |
							ItemSetting::UnlockedMetadata |
							ItemSetting::UnlockedAttributes
					),
				}
			};

			let collection_id =
				T::NFTCollection::create_collection(&who, &who, &collection_config)?;
			// TODO: add a mapping

			// TODO: CollectionId need Debug and Clone, need PR to Substrate
			Self::deposit_event(Event::CollectionCreated { who: who.clone(), collection_id });

			Ok(())
		}
		#[pallet::call_index(1)]
		#[pallet::weight(0)]
		pub fn mint(
			origin: OriginFor<T>,
			collection: CollectionIdOf<T>
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let config = ItemConfig::default();
			T::NFTCollection::mint_into(
				collection,
				1u32.into(),
				&who,
				&config,
				false,
			)?;

			Ok(())
		}
	}
}
