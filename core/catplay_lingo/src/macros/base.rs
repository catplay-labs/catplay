#[doc(hidden)]
#[macro_export]
macro_rules! __iap1_source {
    (device) => {
        $crate::Iap1Source::Device
    };
    (accessory) => {
        $crate::Iap1Source::Accessory
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __iap1_transaction_policy {
    (permitted) => {
        $crate::TransactionIdPolicy::Permitted
    };
    (prohibited) => {
        $crate::TransactionIdPolicy::Prohibited
    };
    (required) => {
        $crate::TransactionIdPolicy::Required
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __iap1_field_type {
    (length_optional, $ty:ty) => { Option<$ty> };
    (required, $ty:ty) => { $ty };
    (optional, $ty:ty) => { Option<$ty> };
}
