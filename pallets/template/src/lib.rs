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
    pub trait Config: frame_system::Config + timestamp::Config {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        type WeightInfo: WeightInfo;
    }

    // Birthday struct to store date information
    #[derive(Clone, Encode, Decode, TypeInfo, MaxEncodedLen, Debug)]
    pub struct Birthday {
        pub year: u16,
        pub month: u8,
        pub day: u8,
    }

    // Modified Tweet struct to use name instead of AccountId
    #[derive(Clone, Encode, Decode, TypeInfo, MaxEncodedLen, Debug)]
    pub struct Tweet {
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
    pub type Birthdays<T: Config> = StorageDoubleMap<
        _,
        Twox64Concat,
        BoundedVec<u8, ConstU32<256>>,
        Twox64Concat,
        u32,
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
    pub type PasswordHash<T: Config> = StorageDoubleMap<
        _,
        Twox64Concat,
        BoundedVec<u8, ConstU32<256>>,
        Twox64Concat,
        u32,
        [u8; 32],
        OptionQuery
    >;

    // Modified tweet storage to use name as key instead of AccountId
    #[pallet::storage]
    pub type Tweets<T: Config> = StorageDoubleMap<
        _,
        Twox64Concat,
        BoundedVec<u8, ConstU32<256>>, // Name as primary key
        Twox64Concat,
        u32,
        Tweet,
        OptionQuery
    >;

    // Modified tweet count storage to use name
    #[pallet::storage]
    pub type TweetCount<T: Config> = StorageMap<
        _,
        Twox64Concat,
        BoundedVec<u8, ConstU32<256>>, // Name as key
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
            name: BoundedVec<u8, ConstU32<256>>,
            year: u16,
            month: u8,
            day: u8,
        },
        PasswordHashed {
            name: BoundedVec<u8, ConstU32<256>>,
        },
        LoginSuccessful {
            name: BoundedVec<u8, ConstU32<256>>,
            who: T::AccountId,
        },
        LoginFailed {
            name: BoundedVec<u8, ConstU32<256>>,
        },
        TweetCreated {
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
        InvalidCaller,
        TweetNotFound,
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
            Birthdays::<T>::insert(&name, 0, birthday);

            // Hash and store password
            let password_hash = blake2_256(&password[..]);
            PasswordHash::<T>::insert(&name, 0, password_hash);

            // Initialize tweet count
            TweetCount::<T>::insert(&name, 0);

            // Emit events
            Self::deposit_event(Event::NameStored { name: name.clone(), who: who.clone() });
            Self::deposit_event(Event::BirthdayStored { name: name.clone(), year, month, day });
            Self::deposit_event(Event::PasswordHashed { name: name.clone() });

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
            let stored_hash = PasswordHash::<T>::get(&name,0)
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
            let who = ensure_signed(origin)?;

            
            // Verify the user exists and owns this name
            let account = AccountByName::<T>::get(&name)
                .ok_or(Error::<T>::UserNotFound)?;
            ensure!(account == who, Error::<T>::NotAuthorized);

            // Verify password
            let stored_hash = PasswordHash::<T>::get(&name,0 )
                .ok_or(Error::<T>::UserNotFound)?;
            let password_hash = blake2_256(&password[..]);
            ensure!(password_hash == stored_hash, Error::<T>::InvalidCredentials);

            // Get current tweet count for name
            let tweet_id = TweetCount::<T>::get(&name);

            // Get current timestamp
            let now = pallet_timestamp::Pallet::<T>::get();
            let timestamp = now.saturated_into::<u64>();

            // Create and store tweet
            let tweet = Tweet {
                name: name.clone(),
                content,
                timestamp,
            };

            Tweets::<T>::insert(&name, tweet_id, tweet);
            TweetCount::<T>::insert(&name, tweet_id.saturating_add(1));

            Self::deposit_event(Event::TweetCreated { 
                name,
                tweet_id,
                timestamp,
            });

            Ok(())
        }

        #[pallet::call_index(3)]
        #[pallet::weight(<T as pallet::Config>::WeightInfo::get_user_birthday())]
        pub fn get_user_birthday(
            origin: OriginFor<T>,
            name: BoundedVec<u8, ConstU32<256>>,
        ) -> DispatchResult {
            ensure_signed(origin)?;

            let account = AccountByName::<T>::get(&name)
                .ok_or(Error::<T>::UserNotFound)?;

            let birthday = Birthdays::<T>::get(&name, 0)
                .ok_or(Error::<T>::UserNotFound)?;

            log::info!("User {:?} has birthday: {:?}", name, birthday);
            Ok(())
        }

        #[pallet::call_index(4)]
        #[pallet::weight(<T as pallet::Config>::WeightInfo::get_user_tweets())]
        pub fn get_user_tweets(
            origin: OriginFor<T>,
            name: BoundedVec<u8, ConstU32<256>>,
        ) -> DispatchResult {
            ensure_signed(origin)?;

            ensure!(AccountByName::<T>::contains_key(&name), Error::<T>::UserNotFound);

            let tweet_count = TweetCount::<T>::get(&name);
            for tweet_id in 0..tweet_count {
                if let Some(tweet) = Tweets::<T>::get(&name, tweet_id) {
                    log::info!("Tweet ID {}: {:?}", tweet_id, tweet);
                }
            }
            Ok(())
        }

        #[pallet::call_index(5)]
        #[pallet::weight(<T as pallet::Config>::WeightInfo::update_user_name())]
        pub fn update_user_name(
            origin: OriginFor<T>,
            name: BoundedVec<u8, ConstU32<256>>,
            password: BoundedVec<u8, ConstU32<256>>,
            new_name: BoundedVec<u8, ConstU32<256>>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let account = AccountByName::<T>::get(&name)
                .ok_or(Error::<T>::UserNotFound)?;
            ensure!(account == who, Error::<T>::NotAuthorized);

            let stored_hash = PasswordHash::<T>::get(&name, 0)
                .ok_or(Error::<T>::UserNotFound)?;

            let password_hash = blake2_256(&password[..]);
            ensure!(password_hash == stored_hash, Error::<T>::InvalidCredentials);

            // Ensure new name isn't taken
            ensure!(!AccountByName::<T>::contains_key(&new_name), Error::<T>::NameAlreadyTaken);

            // Remove old name mapping
            Names::<T>::remove(&who);
            PasswordHash::<T>::remove(&name, 0);
            AccountByName::<T>::remove(&name);

            // Store new name
            Names::<T>::insert(&who, new_name.clone());
            AccountByName::<T>::insert(&new_name, who.clone());
            PasswordHash::<T>::insert(&new_name, 0, password_hash);

            // Move all tweets to new name
            let tweet_count = TweetCount::<T>::get(&name);
            for tweet_id in 0..tweet_count {
                if let Some(mut tweet) = Tweets::<T>::take(&name, tweet_id) {
                    tweet.name = new_name.clone();
                    Tweets::<T>::insert(&new_name, tweet_id, tweet);
                }
            }

            // Update tweet count
            TweetCount::<T>::remove(&name);
            TweetCount::<T>::insert(&new_name, tweet_count);

            Self::deposit_event(Event::NameStored { name: new_name.clone(), who });
            Self::deposit_event(Event::PasswordHashed { name: new_name.clone() });
            

            Ok(())
        }

        #[pallet::call_index(6)]
        #[pallet::weight(<T as pallet::Config>::WeightInfo::update_password())]
        pub fn update_password(
            origin: OriginFor<T>,
            name: BoundedVec<u8, ConstU32<256>>,
            old_password: BoundedVec<u8, ConstU32<256>>,
            new_password: BoundedVec<u8, ConstU32<256>>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let account = AccountByName::<T>::get(&name)
                .ok_or(Error::<T>::UserNotFound)?;
            ensure!(account == who, Error::<T>::NotAuthorized);

            let stored_hash = PasswordHash::<T>::get(&name, 0)
                .ok_or(Error::<T>::UserNotFound)?;

            let old_hash = blake2_256(&old_password[..]);
            ensure!(old_hash == stored_hash, Error::<T>::InvalidCredentials);

            let new_hash = blake2_256(&new_password[..]);
            PasswordHash::<T>::insert(&name, 0, new_hash);

            Self::deposit_event(Event::PasswordHashed { name });

            Ok(())
        }

        #[pallet::call_index(7)]
        #[pallet::weight(<T as pallet::Config>::WeightInfo::delete_tweet())]
        pub fn delete_tweet(
            origin: OriginFor<T>,
            name: BoundedVec<u8, ConstU32<256>>,
            password: BoundedVec<u8, ConstU32<256>>,
            tweet_id: u32,
        ) -> DispatchResult {

            let who = ensure_signed(origin)?;

            // Verify the user exists and owns this name
            let account = AccountByName::<T>::get(&name)
                .ok_or(Error::<T>::UserNotFound)?;
            ensure!(account == who, Error::<T>::NotAuthorized);

            // Verify password
            let stored_hash = PasswordHash::<T>::get(&name, 0)
                .ok_or(Error::<T>::UserNotFound)?;
            let password_hash = blake2_256(&password[..]);
            ensure!(password_hash == stored_hash, Error::<T>::InvalidCredentials);

            // Ensure tweet exists
            ensure!(Tweets::<T>::contains_key(&name, tweet_id), Error::<T>::TweetNotFound);

            // Remove the tweet
            Tweets::<T>::remove(&name, tweet_id);

            Ok(())

        }

        #[pallet::call_index(8)]
        #[pallet::weight(<T as pallet::Config>::WeightInfo::delete_user())]
        pub fn delete_user(
            origin: OriginFor<T>,
            name: BoundedVec<u8, ConstU32<256>>,
            password: BoundedVec<u8, ConstU32<256>>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // Get account by name
            let account = AccountByName::<T>::get(&name)
                .ok_or(Error::<T>::UserNotFound)?;
            
            // Ensure the caller owns this account
            ensure!(account == who, Error::<T>::NotAuthorized);

            // Verify password
            let stored_hash = PasswordHash::<T>::get(&name, 0)
                .ok_or(Error::<T>::UserNotFound)?;
            let password_hash = blake2_256(&password[..]);
            ensure!(password_hash == stored_hash, Error::<T>::InvalidCredentials);

            // Remove name mappings
            Names::<T>::remove(&account);
            AccountByName::<T>::remove(&name);

            // Remove birthday and password
            Birthdays::<T>::remove(&name, 0);
            PasswordHash::<T>::remove(&name, 0);

            // Remove all tweets
            let tweet_count = TweetCount::<T>::get(&name);
            for tweet_id in 0..tweet_count {
                Tweets::<T>::remove(&name, tweet_id);
            }
            TweetCount::<T>::remove(&name);


            Ok(())
        }

    }
}