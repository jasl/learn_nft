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

use frame_support::traits::Currency;
use pallet_nfts::{
	MintType, CollectionSettings, CollectionSetting, ItemSettings, ItemSetting,
	ItemConfig,
	Incrementable,
};

pub type BalanceOf<T, I = ()> =
	<<T as pallet_nfts::Config<I>>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

pub type CollectionIdOf<T, I = ()> = <T as pallet_nfts::Config<I>>::CollectionId;
pub type ItemIdOf<T, I = ()> = <T as pallet_nfts::Config<I>>::ItemId;
pub type CollectionDepositOf<T, I = ()> = <T as pallet_nfts::Config<I>>::CollectionDeposit;
pub type CollectionConfigOf<T, I = ()> = pallet_nfts::CollectionConfig<
	BalanceOf<T, I>,
	<T as frame_system::Config>::BlockNumber,
	CollectionIdOf<T, I>
>;
pub type MintSettingsOf<T, I = ()> = pallet_nfts::MintSettings<
	BalanceOf<T, I>,
	<T as frame_system::Config>::BlockNumber,
	CollectionIdOf<T, I>
>;

pub type PalletNFT<T, I = ()> = pallet_nfts::Pallet<T, I>;

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
	pub struct Pallet<T, I = ()>(_);

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config<I: 'static = ()>: frame_system::Config + pallet_nfts::Config<I> {
		/// Because this pallet emits events, it depends on the runtime definition of an event.
		type RuntimeEvent: From<Event<Self, I>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config<I>, I: 'static = ()> {
		CollectionCreated { worker: T::AccountId, collection_id: CollectionIdOf<T, I> },
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T, I = ()> {
		NotTheOwner,
		WorkerNotExists,
	}

	#[pallet::call]
	impl<T: Config<I>, I: 'static> Pallet<T, I> {
		#[pallet::call_index(0)]
		#[pallet::weight(0)]
		pub fn create_collection(
			origin: OriginFor<T>,
			worker: T::AccountId
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let collection_config = CollectionConfigOf::<T, I> {
				settings: CollectionSettings::from_disabled(
					CollectionSetting::TransferableItems |
						CollectionSetting::UnlockedMetadata |
						CollectionSetting::UnlockedAttributes |
						CollectionSetting::UnlockedMaxSupply
				),
				max_supply: None,
				mint_settings: MintSettingsOf::<T, I> {
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

			let collection =
				pallet_nfts::NextCollectionId::<T, I>::get().unwrap_or(CollectionIdOf::<T, I>::initial_value());

			pallet_nfts::Pallet::<T, I>::do_create_collection(
				collection,
				who.clone(),
				who.clone(),
				collection_config,
				CollectionDepositOf::<T, I>::get(),
				pallet_nfts::Event::<T, I>::Created { collection, creator: who.clone(), owner: who.clone() },
			)?;

			Ok(())
		}

		#[pallet::call_index(1)]
		#[pallet::weight(0)]
		pub fn mint(
			origin: OriginFor<T>,
			collection_id: CollectionIdOf<T, I>
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// let item_id: ItemIdOf<T, I> = 0u32;
			let item_config = ItemConfig {
				settings: ItemSettings::from_disabled(
					ItemSetting::Transferable | ItemSetting::UnlockedMetadata
				)
			};

			pallet_nfts::Pallet::<T, I>::do_mint(
				collection_id,
				0u32.into(),
				Some(who.clone()),
				who.clone(),
				item_config,
				|collection_details, collection_config| {
					// // Issuer can mint regardless of mint settings
					// if Self::has_role(&collection, &caller, CollectionRole::Issuer) {
					// 	return Ok(())
					// }
					//
					// let mint_settings = collection_config.mint_settings;
					// let now = frame_system::Pallet::<T>::block_number();
					//
					// if let Some(start_block) = mint_settings.start_block {
					// 	ensure!(start_block <= now, Error::<T, I>::MintNotStarted);
					// }
					// if let Some(end_block) = mint_settings.end_block {
					// 	ensure!(end_block >= now, Error::<T, I>::MintEnded);
					// }
					//
					// match mint_settings.mint_type {
					// 	MintType::Issuer => return Err(Error::<T, I>::NoPermission.into()),
					// 	MintType::HolderOf(collection_id) => {
					// 		let MintWitness { owner_of_item } =
					// 			witness_data.ok_or(Error::<T, I>::BadWitness)?;
					//
					// 		let has_item = Account::<T, I>::contains_key((
					// 			&caller,
					// 			&collection_id,
					// 			&owner_of_item,
					// 		));
					// 		ensure!(has_item, Error::<T, I>::BadWitness);
					//
					// 		let attribute_key = Self::construct_attribute_key(
					// 			PalletAttributes::<T::CollectionId>::UsedToClaim(collection)
					// 				.encode(),
					// 		)?;
					//
					// 		let key = (
					// 			&collection_id,
					// 			Some(owner_of_item),
					// 			AttributeNamespace::Pallet,
					// 			&attribute_key,
					// 		);
					// 		let already_claimed = Attribute::<T, I>::contains_key(key.clone());
					// 		ensure!(!already_claimed, Error::<T, I>::AlreadyClaimed);
					//
					// 		let value = Self::construct_attribute_value(vec![0])?;
					// 		Attribute::<T, I>::insert(
					// 			key,
					// 			(value, AttributeDeposit { account: None, amount: Zero::zero() }),
					// 		);
					// 	},
					// 	_ => {},
					// }

					// if let Some(price) = mint_settings.price {
					// 	T::Currency::transfer(
					// 		&caller,
					// 		&collection_details.owner,
					// 		price,
					// 		ExistenceRequirement::KeepAlive,
					// 	)?;
					// }

					Ok(())
				},
			)?;

			Ok(())
		}
	}
}
