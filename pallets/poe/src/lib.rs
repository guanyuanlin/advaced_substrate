#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests; // filename

#[frame_support::pallet] // 宏
pub mod pallet {

use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use sp_std::prelude::*;

	// 定义模块需要配置
	#[pallet::config]
	pub trait Config:frame_system::Config {
		#[pallet::constant]
		type MaxClaimLength: Get<u32>;
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

	}
	// 定义模块
	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	pub type Proofs<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		BoundedVec<u8, T::MaxClaimLength>,
		(T::AccountId, T::BlockNumber),
	>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		ClaimCreated(T::AccountId, Vec<u8>),
		ClaimRevoked(T::AccountId, Vec<u8>),
	}

	#[pallet::error]
	pub enum Error<T> {
		ProofAlreadyExist,
		ClaimTooLong,
		ClaimNotExist,
		NostClaimOwner,
	}

	#[pallet::hooks]
	impl<T:Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(0)]
		pub fn create_claim(origin: OriginFor<T>, claim: Vec<u8>) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?; // 校验发送方
			let bounded_claim = BoundedVec::<u8, T::MaxClaimLength>::try_from(claim.clone())
				.map_err(|_| Error::<T>::ClaimTooLong)?; // 校验长度

			// 确认当前proof还没被存储
			ensure!(!Proofs::<T>::contains_key(&bounded_claim), Error::<T>::ProofAlreadyExist);
			// 插入键值对
			Proofs::<T>::insert(
				&bounded_claim,
				(sender.clone(), frame_system::Pallet::<T>::block_number()),
			);
			// 触发事件 并返回ok
			Self::deposit_event(Event::ClaimCreated(sender, claim));
			Ok(().into())
		}
		#[pallet::weight(0)]
		pub fn revoke_claim(
			origin: OriginFor<T>,
			claim: Vec<u8>,
		) -> DispatchResultWithPostInfo {
			// Check that the extrinsic was signed and get the signer.
			// This function will return an error if the extrinsic is not signed.
			// https://substrate.dev/docs/en/knowledgebase/runtime/origin
			let sender = ensure_signed(origin)?;

			let bounded_claim =
			BoundedVec::<u8, <T as Config>::MaxClaimLength>::try_from(claim.clone()).map_err(|_| Error::<T>::ClaimTooLong)?;


			// Verify that the specified proof has been claimed.
			// ensure!(Proofs::<T>::contains_key(&proof), Error::<T>::NoSuchProof);

			// Get owner of the claim.
			let (owner, _) = Proofs::<T>::get(&bounded_claim).ok_or(Error::<T>::ClaimNotExist)?;

			// Verify that sender of the current call is the claim owner.
			ensure!(sender == owner, Error::<T>::NostClaimOwner);

			// Remove claim from storage.
			Proofs::<T>::remove(&bounded_claim);

			// Emit an event that the claim was erased.
			Self::deposit_event(Event::ClaimRevoked(sender, claim));

			Ok(().into())
		}

		#[pallet::weight(0)]
		pub fn transfer_claim(origin: OriginFor<T>, claim: Vec<u8>, dest: T::AccountId) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;
			// u8转BoundedVec
			let bounded_claim =
			BoundedVec::<u8, <T as Config>::MaxClaimLength>::try_from(claim.clone()).map_err(|_| Error::<T>::ClaimTooLong)?;

			// 只有存储在链上的才可以被转移，检查是否在链上
			let (owner, _) = Proofs::<T>::get(&bounded_claim).ok_or(Error::<T>::ClaimNotExist)?;

			ensure!(owner == sender, Error::<T>::NostClaimOwner);

			Proofs::<T>::insert(&bounded_claim, (dest, frame_system::Pallet::<T>::block_number()));
			Ok(().into())
		}
	}
}
