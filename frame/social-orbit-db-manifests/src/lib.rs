#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;


#[frame_support::pallet]
pub mod pallet {
    use pallet_assets::Symbol;
    use frame_support::Blake2_128Concat;
    use frame_support::pallet_prelude::{
        IsType, Hooks, StorageMap, DispatchResultWithPostInfo
    };
    use frame_support::traits::{ Get, Vec };
    use frame_system::pallet_prelude::*;
    // use pallet_society::EnsureFounder;

    // e.g. bafybeig4mpb4myby5gy6n2tfwpsmlykug35ny7v3bdz5qboa5v5on2zmry
    pub type OrbitDBManifestHash = [u8; 58];

    #[pallet::error]
    pub enum Error<T> {
        SocietyIdOverflow
    }

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::event]
    #[pallet::metadata(T::AccountId = "AccountId")]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        ManifestCreated(Symbol, OrbitDBManifestHash, T::AccountId),
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn register_manifest(origin: OriginFor<T>, symbol: Symbol, manifest_hash: OrbitDBManifestHash) -> DispatchResultWithPostInfo {
            let registrant = ensure_signed(origin)?;

            <Manifests<T>>::insert(symbol, &manifest_hash);

            Self::deposit_event(Event::ManifestCreated(symbol, manifest_hash, registrant));

            Ok(().into())
        }

        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn delete_manifest(origin: OriginFor<T>, symbol: Vec<u8>) -> DispatchResultWithPostInfo {
            let _registrant = ensure_signed(origin)?;

            // <Manifests<T>>::remove(&symbol::as_);

            Ok(().into())
        }
    }

    #[pallet::storage]
    #[pallet::getter(fn manifests)]
    pub type Manifests<T: Config> = StorageMap<_, Blake2_128Concat, Symbol, OrbitDBManifestHash>;
}
