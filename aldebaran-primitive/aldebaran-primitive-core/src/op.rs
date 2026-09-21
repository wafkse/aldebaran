//! Generic operations over primitive integer types.

pub mod binary;
pub mod unary;

use self::{binary::BinOp, unary::UnOp};

/// A trait representing a type that can be operated on.
pub trait Operate: BinOp + UnOp {}
/// A trait that represents a cheaply copyable type.
pub trait Operable: Copy + Sized {}

impl<T> Operate for T where T: BinOp + UnOp {}
impl<T> Operable for T where T: Copy + Sized {}

macro_rules! operate_list {
    ($target_macro:path $(=> $lead_tt:tt)?) => {
        $target_macro!(
            $($lead_tt as)?
            Binary {
                Add as +
                Sub as -
                Mul as *
                Div as /
                Rem as %
                BitAnd as &
                BitOr as |
                BitXor as ^
                BitShl try <<
                BitShr try >>
            }
            Unary {
                Not as !
            }
        );
    };
}

macro_rules! impl_list_macros {
    (
        Binary {
            $(
                $target_name:ident $target_mode:ident $target_symbol:tt
            )+
        }
        Unary {
            $(
                $target_name_unary:ident as $target_symbol_unary:tt
            )+
        }
    ) => {
        macro_rules! binary_list {
            ($target_macro:path) => {
                $target_macro!(
                    $($target_name as $target_symbol)+
                );
            };
        }

        macro_rules! unary_list {
            ($target_macro:path) => {
                $target_macro!(
                    $($target_name_unary as $target_symbol_unary)+
                );
            };
        }

        pub(self) use binary_list;
        pub(self) use unary_list;
    };
}

operate_list!(impl_list_macros);

#[allow(unused_imports)]
pub(self) use operate_list;
