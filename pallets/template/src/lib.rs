#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;
use pallet_timestamp as timestamp;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
pub mod weights;
pub use weights::*;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::{pallet_prelude::*, BoundedVec};
    use frame_system::pallet_prelude::*;
    use frame_support::traits::ConstU32;
    use sp_core::hashing::blake2_256;
    use sp_runtime::traits::SaturatedConversion;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config + timestamp::Config{
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        type WeightInfo: WeightInfo;
       
    }

    // Birthday struct to store date information
    #[derive(Clone, Encode, Decode, TypeInfo, MaxEncodedLen)]
    pub struct Birthday {
        pub year: u16,
        pub month: u8,
        pub day: u8,
    }

    // Tweet struct to store tweet data with timestamp
    #[derive(Clone, Encode, Decode, TypeInfo, MaxEncodedLen)]
    pub struct Tweet<AccountId> {
        pub author_id: AccountId,
        pub name: BoundedVec<u8, ConstU32<256>>,
        pub content: BoundedVec<u8, ConstU32<280>>,
        pub timestamp: u64,
    }

    // Store name per account
    #[pallet::storage]
    pub type Names<T: Config> = StorageMap<
        _,
        Twox64Concat,
        T::AccountId,
        BoundedVec<u8, ConstU32<256>>,
        OptionQuery
    >;

    // Store birthday per account
    #[pallet::storage]
    pub type Birthdays<T: Config> = StorageMap<
        _,
        Twox64Concat,
        T::AccountId,
        Birthday,
        OptionQuery
    >;

    // Store name to account mapping for lookups
    #[pallet::storage]
    pub type AccountByName<T: Config> = StorageMap<
        _,
        Twox64Concat,
        BoundedVec<u8, ConstU32<256>>,
        T::AccountId,
        OptionQuery
    >;

    #[pallet::storage]
    pub type PasswordHash<T: Config> = StorageMap<
        _,
        Twox64Concat,
        T::AccountId,
        [u8; 32],
        OptionQuery
    >;

    // Store tweets with a double map: AccountId -> TweetId -> Tweet
    #[pallet::storage]
    pub type Tweets<T: Config> = StorageDoubleMap<
        _,
        Twox64Concat,
        T::AccountId,
        Twox64Concat,
        u32,
        Tweet<T::AccountId>,
        OptionQuery
    >;

    // Store tweet count per user
    #[pallet::storage]
    pub type TweetCount<T: Config> = StorageMap<
        _,
        Twox64Concat,
        T::AccountId,
        u32,
        ValueQuery
    >;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        NameStored {
            name: BoundedVec<u8, ConstU32<256>>,
            who: T::AccountId,
        },
        BirthdayStored {
            who: T::AccountId,
            year: u16,
            month: u8,
            day: u8,
        },
        PasswordHashed {
            who: T::AccountId,
        },
        LoginSuccessful {
            name: BoundedVec<u8, ConstU32<256>>,
            who: T::AccountId,
        },
        LoginFailed {
            name: BoundedVec<u8, ConstU32<256>>,
        },
        TweetCreated {
            who: T::AccountId,
            name: BoundedVec<u8, ConstU32<256>>,
            tweet_id: u32,
            timestamp: u64,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        NoneValue,
        StorageOverflow,
        UserNotFound,
        InvalidCredentials,
        NameAlreadyTaken,
        TweetTooLong,
        NotAuthorized,
        InvalidBirthday,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        #[pallet::weight(<T as pallet::Config>::WeightInfo::create_user())]
        pub fn create_user(
            origin: OriginFor<T>,
            name: BoundedVec<u8, ConstU32<256>>,
            password: BoundedVec<u8, ConstU32<256>>,
            year: u16,
            month: u8,
            day: u8,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // Validate birthday
            ensure!(month <= 12 && month > 0, Error::<T>::InvalidBirthday);
            ensure!(day <= 31 && day > 0, Error::<T>::InvalidBirthday);
            ensure!(year >= 1920 && year <= 2006, Error::<T>::InvalidBirthday);

            // Ensure name isn't already taken
            ensure!(!AccountByName::<T>::contains_key(&name), Error::<T>::NameAlreadyTaken);

            // Store name mappings
            Names::<T>::insert(&who, name.clone());
            AccountByName::<T>::insert(&name, who.clone());

            // Store birthday
            let birthday = Birthday { year, month, day };
            Birthdays::<T>::insert(&who, birthday);

            // Hash and store password
            let password_hash = blake2_256(&password[..]);
            PasswordHash::<T>::insert(&who, password_hash);

            // Initialize tweet count
            TweetCount::<T>::insert(&who, 0);

            // Emit events
            Self::deposit_event(Event::NameStored { name, who: who.clone() });
            Self::deposit_event(Event::BirthdayStored { who: who.clone(), year, month, day });
            Self::deposit_event(Event::PasswordHashed { who });

            Ok(())
        }

        #[pallet::call_index(1)]
        #[pallet::weight(<T as pallet::Config>::WeightInfo::verify_user())]
        pub fn verify_user(
            origin: OriginFor<T>,
            name: BoundedVec<u8, ConstU32<256>>,
            password: BoundedVec<u8, ConstU32<256>>
        ) -> DispatchResult {
            let _ = ensure_signed(origin)?;

            // Get account by name
            let account = AccountByName::<T>::get(&name)
                .ok_or(Error::<T>::UserNotFound)?;

            // Get stored password hash
            let stored_hash = PasswordHash::<T>::get(&account)
                .ok_or(Error::<T>::UserNotFound)?;

            // Hash the provided password
            let password_hash = blake2_256(&password[..]);

            // Compare hashes using constant-time comparison
            if password_hash == stored_hash {
                Self::deposit_event(Event::LoginSuccessful { name, who: account });
                Ok(())
            } else {
                Self::deposit_event(Event::LoginFailed { name });
                Err(Error::<T>::InvalidCredentials.into())
            }
        }

        #[pallet::call_index(2)]
        #[pallet::weight(<T as pallet::Config>::WeightInfo::create_tweet())]
        pub fn create_tweet(
            origin: OriginFor<T>,
            name: BoundedVec<u8, ConstU32<256>>,
            password: BoundedVec<u8, ConstU32<256>>,
            content: BoundedVec<u8, ConstU32<280>>
        ) -> DispatchResult {
            let account = AccountByName::<T>::get(&name)
                .ok_or(Error::<T>::UserNotFound)?;

            // Get stored password hash
            let stored_hash = PasswordHash::<T>::get(&account)
                .ok_or(Error::<T>::UserNotFound)?;
            // Hash the provided password
            let password_hash = blake2_256(&password[..]);

            let who = ensure_signed(origin)?;

            // Get current tweet count for user
            let tweet_id = TweetCount::<T>::get(&who);

            // Get current block number and convert it to u64 timestamp
            let now =  pallet_timestamp::Pallet::<T>::get();
            let timestamp = now.saturated_into::<u64>();

            // Create tweet with timestamp
            let tweet = Tweet {
                author_id: who.clone(),
                name: name.clone(),
                content,
                timestamp,
            };

            if password_hash == stored_hash {
                // Store tweet
                Tweets::<T>::insert(&who, tweet_id, tweet);

                // Increment tweet count
                TweetCount::<T>::insert(&who, tweet_id.saturating_add(1));

                Self::deposit_event(Event::TweetCreated { 
                    who, 
                    name,
                    tweet_id,
                    timestamp,
                });
                Ok(())
            } else {
                Self::deposit_event(Event::LoginFailed { name });
                Err(Error::<T>::InvalidCredentials.into())
            }
        }
    }
}