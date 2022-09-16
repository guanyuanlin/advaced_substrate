#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::traits::{
		tokens::ExistenceRequirement, Currency, Randomness, ReservableCurrency,
	};
	use frame_support::{pallet_prelude::*, Twox64Concat};
	use frame_system::pallet_prelude::*;
	use scale_info::TypeInfo;
	use sp_io::hashing::blake2_128;
	use sp_runtime::traits::{AtLeast32BitUnsigned, Bounded, One};

	#[cfg(feature = "std")]
	use frame_support::serde::{Deserialize, Serialize};

	type AccountOf<T> = <T as frame_system::Config>::AccountId;
	type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	#[scale_info(skip_type_params(T))]
	#[codec(mel_bound())]
	pub struct Kitty<T: Config> {
		pub dna: [u8; 16], // 为每个kitty 属性分配存储空间， 16个u8 数据存储来保存它
		pub price: Option<BalanceOf<T>>,
		pub gender: Gender,
		pub owner: AccountOf<T>,
	}
	#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
	pub enum Gender {
		Male,
		Female,
	}

	// #[pallet::type_value]
	// pub fn GetDefaultValue() -> KittyIndex {
	// 	// 定义结构的默认值获取方式
	// 	0_u32
	// }

	// #[derive(Encode, Clone, Debug, PartialEq, Eq, Decode, TypeInfo, MaxEncodedLen)]
	// pub struct Kitty(pub [u8; 16]);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// The type of Randomness we want to specify for this pallet.
		type KittyRandomness: Randomness<Self::Hash, Self::BlockNumber>;
		/// The maximum amount of Kitties a single account can own.
		#[pallet::constant]
		type MaxKittyOwned: Get<u32>;

		/// 引入资产类型，以便支持质押
		/// 参考：substrate/frame/treasury/src/lib.rs中的定义
		type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;

		// Storage items. 小猫序号
		// 定义KittyIndex类型，要求实现执行的trait
		// Paramter 表示可以用于函数参数传递
		// AtLeast32Bit 表示转换为u32不会造成数据丢失
		// Default 表示有默认值
		// Copy 表示实现Copy方法
		// Bounded 表示包含上界和下界
		// 以后开发遇到在Runtime中定义无符号整型，可以直接复制套用
		type KittyIndex: Parameter + AtLeast32BitUnsigned + Default + Copy + Bounded + MaxEncodedLen;

		// 定义常量时，必须带上以下宏
		// 获取Runtime中Kitties pallet定义的质押金额常量
		// 在创建Kitty前需要做质押，避免反复恶意创建
		#[pallet::constant]
		type KittyStake: Get<BalanceOf<Self>>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_); // ??? 这个是啥意思

	// type KittyIndex = u32; // kitty 标识

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		// KittyCreated(T::AccountId, KittyIndex, Kitty),
		// KittyBred(T::AccountId, KittyIndex, Kitty),
		// KittyTransferred(T::AccountId, T::AccountId, KittyIndex),

		// ACTION #3: Declare events
		/// A new Kitty was successfully created. \[sender, kitty_id\]
		Created(T::AccountId, T::KittyIndex),
		/// Kitty price was successfully set. \[sender, kitty_id, new_price\]
		PriceSet(T::AccountId, T::KittyIndex, Option<BalanceOf<T>>),
		/// A Kitty was successfully transferred. \[from, to, kitty_id\]
		Transferred(T::AccountId, T::AccountId, T::KittyIndex),
		/// A Kitty was successfully bought. \[buyer, seller, kitty_id, bid_price\]
		Bought(T::AccountId, T::AccountId, T::KittyIndex, BalanceOf<T>),
		/// A Kitty was successfully bred. \[sender, new_kitty_id, parent1, parent2\],
		BredSuccess(T::AccountId, T::KittyIndex, T::KittyIndex, T::KittyIndex),
	}

	#[pallet::error]
	pub enum Error<T> {
		InvalidKittyId,
		SameKittyId,

		// ACTION #5a: Declare errors.
		/// Handles arithmetic overflow when incrementing the Kitty counter.
		KittyIndexOverflow,
		/// An account cannot own more Kitties than `MaxKittyCount`.
		ExceedMaxKittyOwned,
		/// Buyer cannot be the owner.
		BuyerIsKittyOwner,
		/// Cannot transfer a kitty to its owner.
		TransferToSelf,
		/// Handles checking whether the Kitty exists.
		KittyNotExist,
		KittyExists, //Kitty exists.

		/// Handles checking that the Kitty is owned by the account transferring, buying or setting a price for it.    
		NotKittyOwner,
		/// Ensures the Kitty is for sale.
		KittyNotForSale,
		/// Ensures that the buying price is greater than the asking price.
		KittyBidPriceTooLow,
		/// Ensures that an account has enough funds to purchase a Kitty.
		NotEnoughBalance,
	} //ValueQuery ???

	// #[pallet::storage]
	// #[pallet::getter(fn next_kitty_id)]
	// pub type NextKittyId<T> = StorageValue<_, KittyIndex, ValueQuery, GetDefaultValue>;

	// #[pallet::storage]
	// #[pallet::getter(fn kitties)]
	// pub type Kittys<T> = StorageMap<_, Blake2_128Concat, KittyIndex, Kitty>;

	// #[pallet::storage]
	// #[pallet::getter(fn kitty_owner)]
	// pub type KittyOwner<T: Config> = StorageMap<_, Blake2_128Concat, KittyIndex, T::AccountId>;

	// 定义存储
	#[pallet::storage]
	// getter声明外部 要查询存储时，可以调用get_kitty_id方法，方法名称可自定义。
	#[pallet::getter(fn last_kitty_id)]
	/// 存储kitty最新的id，用作索引，也可以用作kitty数量总计(+1)
	pub type LastKittyId<T: Config> = StorageValue<_, T::KittyIndex, ValueQuery>; // KittyIndex移到Runtime后，KittyIndex改为T::KittyIndex

	#[pallet::storage]
	#[pallet::getter(fn kitties)]
	/// Stores a Kitty's unique traits, owner and price.
	pub(super) type Kitties<T: Config> = StorageMap<_, Twox64Concat, T::KittyIndex, Kitty<T>>;

	#[pallet::storage]
	#[pallet::getter(fn kitties_owned)]
	/// Keeps track of what accounts own what Kitty.
	pub(super) type KittiesOwned<T: Config> = StorageMap<
		_,
		Twox64Concat,
		T::AccountId,
		BoundedVec<T::KittyIndex, T::MaxKittyOwned>,
		ValueQuery,
	>;

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(100)]
		pub fn create_kitty(origin: OriginFor<T>) -> DispatchResult {
			// ACTION #1: create_kitty
			let sender = ensure_signed(origin)?;

			// 获取需要质押的金额
			let stake_amount = T::KittyStake::get();

			// 质押制定数量的资产，如果质押失败则报错
			T::Currency::reserve(&sender, stake_amount)
				.map_err(|_| Error::<T>::NotEnoughBalance)?;

			let kitty_id = Self::mint(&sender, None, None)?;
			// Logging to the console
			log::info!("A kitty is born with ID: {:?}", kitty_id);

			// ACTION #4: Deposit `Created` event
			Self::deposit_event(Event::Created(sender, kitty_id));
			// let kitty_id = Self::get_next_id().map_err(|_| Error::<T>::InvalidKittyId)?;

			// let dna = Self::random_value(&who);
			// let kitty = Kitty(dna);
			// Kittys::<T>::insert(kitty_id, &kitty);
			// KittyOwner::<T>::insert(kitty_id, &who);
			// NextKittyId::<T>::set(kitty_id + 1);

			// // Emit an event
			// Self::deposit_event(Event::KittyCreated(who, kitty_id, kitty));
			Ok(())
		}

		// TODO Part IV: set_price
		#[pallet::weight(100)] // 10_1000
		pub fn set_price(
			origin: OriginFor<T>,
			kitty_id: T::KittyIndex,
			new_price: Option<BalanceOf<T>>,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			// ACTION #1a: Checking Kitty owner
			ensure!(Self::is_kitty_owner(kitty_id, &sender)?, <Error<T>>::NotKittyOwner);
			let mut kitty = Self::kitties(kitty_id).ok_or(<Error<T>>::KittyNotExist)?;

			// ACTION #2: Set the Kitty price and update new Kitty infomation to storage.
			kitty.price = new_price.clone();
			<Kitties<T>>::insert(kitty_id, kitty);

			// ACTION #3: Deposit a "PriceSet" event.
			Self::deposit_event(Event::PriceSet(sender, kitty_id, new_price));

			Ok(())
		}

		#[pallet::weight(1000)] // 10_1000
		pub fn transfer(
			origin: OriginFor<T>,
			to: T::AccountId,
			kitty_id: T::KittyIndex,
		) -> DispatchResult {
			let from = ensure_signed(origin)?;

			// Ensure the kitty exists and is called by the kitty owner.
			ensure!(Self::is_kitty_owner(kitty_id, &from)?, <Error<T>>::NotKittyOwner);

			// Verify the kitty is not transferring back to its owner.
			ensure!(from != to, <Error<T>>::TransferToSelf);

			// Verify the the recipient has the capacity to receive one more kitty.
			let to_owned = <KittiesOwned<T>>::get(&to);
			ensure!(
				(to_owned.len() as u32) < T::MaxKittyOwned::get(),
				<Error<T>>::ExceedMaxKittyOwned
			);
			// 获取需要质押的金额
			let stake_amount = T::KittyStake::get();

			// 新的Owner账户进行质押
			T::Currency::reserve(&to, stake_amount).map_err(|_| Error::<T>::NotEnoughBalance)?;
			// 旧的Owner账户解除质押
			T::Currency::unreserve(&from, stake_amount);

			Self::transfer_kitty_to(kitty_id, &to)?;
			Self::deposit_event(Event::Transferred(from, to, kitty_id));

			Ok(())
		}

		#[pallet::weight(100)] // 10_1000
		pub fn buy_kitty(
			origin: OriginFor<T>,
			kitty_id: T::KittyIndex,
			bid_price: BalanceOf<T>,
		) -> DispatchResult {
			let buyer = ensure_signed(origin)?;

			// Check the kitty exists and buyer is not the current kitty owner
			let kitty = Self::kitties(&kitty_id).ok_or(<Error<T>>::KittyNotExist)?;
			ensure!(kitty.owner != buyer, <Error<T>>::BuyerIsKittyOwner);

			// ACTION #7: Check if buyer can receive Kitty.
			// Verify the buyer has the capacity to receive one more kitty
			let to_owned = <KittiesOwned<T>>::get(&buyer);
			ensure!(
				(to_owned.len() as u32) < T::MaxKittyOwned::get(),
				<Error<T>>::ExceedMaxKittyOwned
			);
			let seller = kitty.owner.clone();
			// ACTION #6: Check if the Kitty is for sale.
			// Check the kitty is for sale and the kitty ask price <= bid_price
			if let Some(ask_price) = kitty.price {
				ensure!(ask_price <= bid_price, <Error<T>>::KittyBidPriceTooLow);
			} else {
				Err(<Error<T>>::KittyNotForSale)?;
			}
			// 获取需要质押的金额配置
			let stake_amount = T::KittyStake::get();
			// 检查买家的余额是否足够用于购买和质押
			let buyer_balance = T::Currency::free_balance(&buyer);
			ensure!(buyer_balance > (bid_price + stake_amount), <Error<T>>::NotEnoughBalance);

			// 买家质押
			T::Currency::reserve(&buyer, stake_amount).map_err(|_| Error::<T>::NotEnoughBalance)?;
			// 卖家解除质押
			T::Currency::unreserve(&seller, stake_amount);
			// Check the buyer has enough free balance
			ensure!(T::Currency::free_balance(&buyer) >= bid_price, <Error<T>>::NotEnoughBalance);

			// ACTION #8: Update Balances using the Currency trait.
			// Transfer the amount from buyer to seller
			T::Currency::transfer(&buyer, &seller, bid_price, ExistenceRequirement::KeepAlive)?;
			// Transfer the kitty from seller to buyer
			Self::transfer_kitty_to(kitty_id, &buyer)?;
			// Deposit relevant Event
			Self::deposit_event(Event::Bought(buyer, seller, kitty_id, bid_price));
			Ok(())
		}

		#[pallet::weight(100)] // 10_1000
		pub fn breed_kitty(
			origin: OriginFor<T>,
			parent1: T::KittyIndex,
			parent2: T::KittyIndex,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			// 获取需要质押的金额
			let stake_amount = T::KittyStake::get();

			// 质押指定数量的资产，如果资产质押失败则报错
			T::Currency::reserve(&sender, stake_amount)
				.map_err(|_| Error::<T>::NotEnoughBalance)?;

			// Check: Verify `sender` owns both kitties (and both kitties exist).
			ensure!(Self::is_kitty_owner(parent1, &sender)?, <Error<T>>::NotKittyOwner);
			ensure!(Self::is_kitty_owner(parent2, &sender)?, <Error<T>>::NotKittyOwner);

			// ACTION #9: Breed two Kitties using unique DNA
			let new_dna = Self::breed_dna(parent1, parent2)?;

			// ACTION #10: Mint new Kitty using new DNA
			let kitty_id = Self::mint(&sender, Some(new_dna), None)?;

			// Deposit relevant Event
			Self::deposit_event(Event::BredSuccess(sender, kitty_id, parent1, parent2));

			Ok(())
		}
		// #[pallet::weight(100)] // 10_1000
		// pub fn breed(
		// 	origin: OriginFor<T>,
		// 	kitty_id_1: KittyIndex,
		// 	kitty_id_2: KittyIndex,
		// ) -> DispatchResult {
		// 	let who = ensure_signed(origin)?;

		// 	// check kitty id
		// 	ensure!(kitty_id_1 != kitty_id_2, Error::<T>::SameKittyId);
		// 	let kitty_1 = Self::get_kitty(kitty_id_1).map_err(|_| Error::<T>::InvalidKittyId)?;
		// 	let kitty_2 = Self::get_kitty(kitty_id_2).map_err(|_| Error::<T>::InvalidKittyId)?;

		// 	// get next id
		// 	let kitty_id = Self::get_next_id().map_err(|_| Error::<T>::InvalidKittyId)?;

		// 	// selector for breeding
		// 	let selector = Self::random_value(&who);

		// 	let mut data = [0u8; 16];
		// 	for i in 0..kitty_1.0.len() {
		// 		// 0 choose kitty2, and 1 choose kitty1
		// 		data[i] = (kitty_1.0[i] & selector[i]) | (kitty_2.0[i] & !selector[i]);
		// 	}
		// 	let new_kitty = Kitty(data);

		// 	<Kittys<T>>::insert(kitty_id, &new_kitty);
		// 	KittyOwner::<T>::insert(kitty_id, &who);
		// 	NextKittyId::<T>::set(kitty_id + 1);

		// 	Self::deposit_event(Event::KittyCreated(who, kitty_id, new_kitty));
		// 	Ok(())
		// }

		// #[pallet::weight(10_000)]
		// pub fn transfer(
		// 	origin: OriginFor<T>,
		// 	kitty_id: u32,
		// 	new_owner: T::AccountId,
		// ) -> DispatchResult {
		// 	let who = ensure_signed(origin)?;

		// 	Self::get_kitty(kitty_id).map_err(|_| Error::<T>::InvalidKittyId)?;

		// 	ensure!(Self::kitty_owner(kitty_id) == Some(who.clone()), Error::<T>::NotOwner);

		// 	<KittyOwner<T>>::insert(kitty_id, new_owner);

		// 	Ok(())
		// }
	}

	impl<T: Config> Pallet<T> {
		// get next id
		fn get_next_id() -> Result<T::KittyIndex, Error<T>> {
			let kitty_id = Self::last_kitty_id() + One::one();
			if kitty_id == T::KittyIndex::max_value() {
				return Err(Error::<T>::KittyIndexOverflow);
			}
			return Ok(kitty_id);
			// match Self::next_kitty_id() {
			// 	T::KittyIndex::max_value() => Err(Error::<T>::KittyIndexOverflow),
			// 	val => Ok(val),
			// }
		}

		// get kitty via id
		fn get_kitty(kitty_id: T::KittyIndex) -> Result<Kitty<T>, Error<T>> {
			match Self::kitties(kitty_id) {
				Some(kitty) => Ok(kitty),
				None => Err(Error::<T>::KittyNotExist),
			}
		}

		// Generate a random gender value
		fn gen_gender() -> Gender {
			let random = T::KittyRandomness::random(&b"gender"[..]).0;
			match random.as_ref()[0] % 2 {
				0 => Gender::Male,
				_ => Gender::Female,
			}
		}

		// Generate a random DNA value
		fn gen_dna() -> [u8; 16] {
			let payload = (
				T::KittyRandomness::random(&b"dna"[..]).0,
				<frame_system::Pallet<T>>::extrinsic_index().unwrap_or_default(),
				<frame_system::Pallet<T>>::block_number(),
			);
			payload.using_encoded(blake2_128)
		}

		// Create new DNA with existing DNA
		pub fn breed_dna(
			parent1: T::KittyIndex,
			parent2: T::KittyIndex,
		) -> Result<[u8; 16], Error<T>> {
			let dna1 = Self::get_kitty(parent1)?.dna;
			let dna2 = Self::get_kitty(parent2)?.dna;

			let mut new_dna = Self::gen_dna();
			for i in 0..new_dna.len() {
				new_dna[i] = (new_dna[i] & dna1[i]) | (!new_dna[i] & dna2[i]);
			}
			Ok(new_dna)
		}

		// ACTION #2: Write mint function
		// Helper to mint a Kitty.
		pub fn mint(
			owner: &T::AccountId,
			dna: Option<[u8; 16]>,
			gender: Option<Gender>,
		) -> Result<T::KittyIndex, Error<T>> {
			let kitty = Kitty::<T> {
				dna: dna.unwrap_or_else(Self::gen_dna),
				price: None,
				gender: gender.unwrap_or_else(Self::gen_gender),
				owner: owner.clone(),
			};
			let kitty_id = Self::last_kitty_id(); // 当前序号， 从0开始
			let next_id = Self::get_next_id()?; // 递增序号
			ensure!(Self::kitties(&kitty_id) == None, Error::<T>::KittyExists);

			// ??? try_mutate???
			// Perform this operation first because as it may fail
			<KittiesOwned<T>>::try_mutate(&owner, |kitty_vec| kitty_vec.try_push(kitty_id))
				.map_err(|_| Error::<T>::ExceedMaxKittyOwned)?;

			<Kitties<T>>::insert(kitty_id, kitty);
			<LastKittyId<T>>::put(next_id);
			Ok(kitty_id)
		}

		pub fn is_kitty_owner(
			kitty_id: T::KittyIndex,
			acct: &T::AccountId,
		) -> Result<bool, Error<T>> {
			match Self::kitties(kitty_id) {
				Some(kitty) => Ok(kitty.owner == *acct),
				None => Err(<Error<T>>::KittyNotExist),
			}
		}

		pub fn transfer_kitty_to(
			kitty_id: T::KittyIndex,
			to: &T::AccountId,
		) -> Result<(), Error<T>> {
			let mut kitty = Self::kitties(kitty_id).ok_or(<Error<T>>::KittyNotExist)?;
			let prev_owner = kitty.owner.clone();

			// Remove `kitty_id` from the KittyOwned vector of `prev_kitty_owner`.
			<KittiesOwned<T>>::try_mutate(&prev_owner, |owned| {
				if let Some(ind) = owned.iter().position(|&id| id == kitty_id) {
					owned.swap_remove(ind);
					return Ok(());
				}
				Err(())
			})
			.map_err(|_| Error::<T>::KittyNotExist)?;

			// Update the kitty owner
			kitty.owner = to.clone();

			// Reset the ask price so the kitty is not for sale until `set_price()` is called
			// by the current owner.
			kitty.price = None;

			<Kitties<T>>::insert(kitty_id, kitty);
			<KittiesOwned<T>>::try_mutate(to, |vec| vec.try_push(kitty_id))
				.map_err(|_| Error::<T>::ExceedMaxKittyOwned)?;
			Ok(())
		}
	}
}
