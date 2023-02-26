use crate::{FromHciBytes, FromHciBytesError, WriteHci};

macro_rules! impl_param_int {
    ($($ty:ty),+) => {
        $(
            impl WriteHci for $ty {
                fn size(&self) -> usize {
                    ::core::mem::size_of::<Self>()
                }

                fn write_hci<W: ::embedded_io::blocking::Write>(&self, mut writer: W) -> Result<(), W::Error> {
                    writer.write_all(&self.to_le_bytes())
                }

                #[cfg(feature = "async")]
                async fn write_hci_async<W: ::embedded_io::asynch::Write>(&self, mut writer: W) -> Result<(), W::Error> {
                    writer.write_all(&self.to_le_bytes()).await
                }
            }

            impl<'de> FromHciBytes<'de> for $ty {
                fn from_hci_bytes(data: &'de [u8]) -> Result<(Self, &'de [u8]), FromHciBytesError> {
                    let size = ::core::mem::size_of::<Self>();
                    if data.len() >= size {
                        let (data, rest) = data.split_at(size);
                        Ok((Self::from_le_bytes(unsafe { data.try_into().unwrap_unchecked() }), rest))
                    } else {
                        Err($crate::FromHciBytesError::InvalidSize)
                    }

                }
            }
        )+
    };
}

impl_param_int!(u8, i8, u16, i16, u32);

macro_rules! impl_param_tuple {
    ($($a:ident)*) => {
        #[automatically_derived]
        #[allow(non_snake_case)]
        impl<$($a: WriteHci,)*> WriteHci for ($($a,)*) {
            fn size(&self) -> usize {
                let ($(ref $a,)*) = *self;
                $($a.size() +)* 0
            }

            #[allow(unused_mut, unused_variables)]
            fn write_hci<W: ::embedded_io::blocking::Write>(&self, mut writer: W) -> Result<(), W::Error> {
                let ($(ref $a,)*) = *self;
                $($a.write_hci(&mut writer)?;)*
                Ok(())
            }

            #[cfg(feature = "async")]
            #[allow(unused_mut, unused_variables)]
            async fn write_hci_async<W: ::embedded_io::asynch::Write>(&self, mut writer: W) -> Result<(), W::Error> {
                let ($(ref $a,)*) = *self;
                $($a.write_hci_async(&mut writer).await?;)*
                Ok(())
            }
        }

        #[automatically_derived]
        #[allow(non_snake_case)]
        impl<'de, $($a: FromHciBytes<'de>,)*> FromHciBytes<'de> for ($($a,)*) {
            #[allow(unused_mut, unused_variables)]
            fn from_hci_bytes(data: &'de [u8]) -> Result<(Self, &'de [u8]), FromHciBytesError> {
                let total = 0;
                $(
                    let ($a, data) = $a::from_hci_bytes(data)?;
                )*
                Ok((($($a,)*), data))
            }
        }
    };
}

impl_param_tuple! {}
impl_param_tuple! { A B }

macro_rules! param {
    (struct $name:ident($wrapped:ty)) => {
        $crate::param::param! {
            #[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
            struct $name($wrapped);
        }
    };
    (
        #[derive($($derive:ty),*)]
        struct $name:ident($wrapped:ty);
    ) => {
        #[repr(transparent)]
        #[derive($($derive,)*)]
        #[cfg_attr(feature = "defmt", derive(defmt::Format))]
        pub struct $name($wrapped);

        impl $name {
            pub fn into_inner(self) -> $wrapped {
                self.0
            }
        }

        impl $crate::WriteHci for $name {
            fn size(&self) -> usize {
                $crate::WriteHci::size(&self.0)
            }

            fn write_hci<W: ::embedded_io::blocking::Write>(&self, writer: W) -> Result<(), W::Error> {
                <$wrapped as $crate::WriteHci>::write_hci(&self.0, writer)
            }

            #[cfg(feature = "async")]
            async fn write_hci_async<W: ::embedded_io::asynch::Write>(&self, writer: W) -> Result<(), W::Error> {
                <$wrapped as $crate::WriteHci>::write_hci_async(&self.0, writer).await
            }
        }

        impl<'de> $crate::FromHciBytes<'de> for $name {
            fn from_hci_bytes(data: &'de [u8]) -> Result<(Self, &'de [u8]), $crate::FromHciBytesError> {
                <$wrapped as $crate::FromHciBytes>::from_hci_bytes(data).map(|(x, y)| (Self(x), y))
            }
        }
    };

    (struct $name:ident {
        $($field:ident: $ty:ty),*
        $(,)?
    }) => {
        $crate::param::param! {
            #[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
            struct $name {
                $($field: $ty,)*
            }
        }
    };
    (
        #[derive($($derive:ty),*)]
        struct $name:ident {
            $($field:ident: $ty:ty),*
            $(,)?
        }
    ) => {
        #[derive($($derive,)*)]
        #[cfg_attr(feature = "defmt", derive(defmt::Format))]
        pub struct $name {
            pub $($field: $ty,)*
        }

        impl $crate::WriteHci for $name {
            fn size(&self) -> usize {
                $(<$ty as $crate::WriteHci>::size(&self.$field) +)* 0
            }

            fn write_hci<W: ::embedded_io::blocking::Write>(&self, mut writer: W) -> Result<(), W::Error> {
                $(<$ty as $crate::WriteHci>::write_hci(&self.$field, &mut writer)?;)*
                Ok(())
            }

            #[cfg(feature = "async")]
            async fn write_hci_async<W: ::embedded_io::asynch::Write>(&self, mut writer: W) -> Result<(), W::Error> {
                $(<$ty as $crate::WriteHci>::write_hci_async(&self.$field, &mut writer).await?;)*
                Ok(())
            }
        }

        impl<'de> $crate::FromHciBytes<'de> for $name {
            #[allow(unused_variables)]
            fn from_hci_bytes(data: &'de [u8]) -> Result<(Self, &'de [u8]), $crate::FromHciBytesError> {
                let total = 0;
                $(
                    let ($field, data) = <$ty as $crate::FromHciBytes>::from_hci_bytes(data)?;
                )*
                Ok((Self {
                    $($field,)*
                }, data))
            }
        }
    };

    (
        enum $name:ident {
            $(
                $variant:ident = $value:expr,
            )+
        }
    ) => {
        $crate::param::param! {
            #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
            enum $name {
                $($variant = $value,)+
            }
        }
    };
    (
        #[derive($($derive:ty),* $(,)?)]
        enum $name:ident {
            $(
                $variant:ident = $value:expr,
            )+
        }
    ) => {
        #[repr(u8)]
        #[derive($($derive,)*)]
        #[cfg_attr(feature = "defmt", derive(defmt::Format))]
        pub enum $name {
            $(
                $variant = $value,
            )+
        }

        impl $crate::WriteHci for $name {
            fn size(&self) -> usize {
                1
            }

            fn write_hci<W: ::embedded_io::blocking::Write>(&self, writer: W) -> Result<(), W::Error> {
                <u8 as $crate::WriteHci>::write_hci(&(*self as u8), writer)
            }

            #[cfg(feature = "async")]
            async fn write_hci_async<W: ::embedded_io::asynch::Write>(&self, writer: W) -> Result<(), W::Error> {
                <u8 as $crate::WriteHci>::write_hci_async(&(*self as u8), writer).await
            }
        }

        impl<'de> $crate::FromHciBytes<'de> for $name {
            #[allow(unused_variables)]
            fn from_hci_bytes(data: &'de [u8]) -> Result<(Self, &'de [u8]), $crate::FromHciBytesError> {
                match data.split_first() {
                    Some((byte, data)) => match byte {
                        $($value => Ok((Self::$variant, data)),)+
                        _ => Err($crate::FromHciBytesError::InvalidValue),
                    }
                    None => Err($crate::FromHciBytesError::InvalidSize),
                }
            }
        }
    };

    (
        bitfield $name:ident[$octets:expr] {
            $(($bit:expr, $get:ident, $set:ident);)+
        }
    ) => {
        $crate::param::param! {
            #[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
            bitfield $name[$octets] {
                $(($bit, $get, $set);)+
            }
        }
    };
    (
        #[derive($($derive:ty),*)]
        bitfield $name:ident[1] {
            $(($bit:expr, $get:ident, $set:ident);)+
        }
    ) => {
        #[repr(transparent)]
        #[derive($($derive,)*)]
        #[cfg_attr(feature = "defmt", derive(defmt::Format))]
        pub struct $name(u8);

        impl $name {
            pub fn into_inner(self) -> u8 {
                self.0
            }

            $(
                pub const fn $get(&self) -> bool {
                    (self.0 & (1 << $bit)) != 0
                }

                pub const fn $set(self, val: bool) -> Self {
                    Self((self.0 & !(1 << $bit)) | ((val as u8) << $bit))
                }
            )+
        }

        impl $crate::WriteHci for $name {
            fn size(&self) -> usize {
                1
            }

            fn write_hci<W: ::embedded_io::blocking::Write>(&self, writer: W) -> Result<(), W::Error> {
                <u8 as $crate::WriteHci>::write_hci(&self.0, writer)
            }

            #[cfg(feature = "async")]
            #[allow(unused_mut)]
            async fn write_hci_async<W: ::embedded_io::asynch::Write>(&self, mut writer: W) -> Result<(), W::Error> {
                <u8 as $crate::WriteHci>::write_hci_async(&self.0, writer).await
            }
        }
    };
    (
        #[derive($($derive:ty),*)]
        bitfield $name:ident[$octets:expr] {
            $(($bit:expr, $get:ident, $set:ident);)+
        }
    ) => {
        #[repr(transparent)]
        #[derive($($derive,)*)]
        #[cfg_attr(feature = "defmt", derive(defmt::Format))]
        pub struct $name([u8; $octets]);

        impl $name {
            pub fn into_inner(self) -> [u8; $octets] {
                self.0
            }

            $(
                pub const fn $get(&self) -> bool {
                    const OCTET: usize = $bit / 8;
                    const BIT: usize = $bit % 8;
                    (self.0[OCTET] & (1 << BIT)) != 0
                }

                pub const fn $set(mut self, val: bool) -> Self {
                    const OCTET: usize = $bit / 8;
                    const BIT: usize = $bit % 8;
                    self.0[OCTET] = (self.0[OCTET] & !(1 << BIT)) | ((val as u8) << BIT);
                    self
                }
            )+
        }

        impl $crate::WriteHci for $name {
            fn size(&self) -> usize {
                $octets
            }

            fn write_hci<W: ::embedded_io::blocking::Write>(&self, writer: W) -> Result<(), W::Error> {
                <[u8; $octets] as $crate::WriteHci>::write_hci(&self.0, writer)
            }

            #[cfg(feature = "async")]
            #[allow(unused_mut)]
            async fn write_hci_async<W: ::embedded_io::asynch::Write>(&self, mut writer: W) -> Result<(), W::Error> {
                <[u8; $octets] as $crate::WriteHci>::write_hci_async(&self.0, writer).await
            }
        }

        impl<'de> $crate::FromHciBytes<'de> for $name {
            fn from_hci_bytes(data: &'de [u8]) -> Result<(Self, &'de [u8]), $crate::FromHciBytesError> {
                <[u8; $octets] as $crate::FromHciBytes>::from_hci_bytes(data).map(|(x,y)| (Self(x), y))
            }
        }
    };

    (&$life:lifetime [$el:ty]) => {
        impl<$life> $crate::WriteHci for &$life [$el] {
            fn size(&self) -> usize {
                1 + self.iter().map($crate::WriteHci::size).sum::<usize>()
            }

            fn write_hci<W: ::embedded_io::blocking::Write>(&self, mut writer: W) -> Result<(), W::Error> {
                writer.write_all(&[self.len() as u8])?;
                for x in self.iter() {
                    <$el as $crate::WriteHci>::write_hci(x, &mut writer)?;
                }
                Ok(())
            }

            #[cfg(feature = "async")]
            async fn write_hci_async<W: ::embedded_io::asynch::Write>(&self, mut writer: W) -> Result<(), W::Error> {
                writer.write_all(&[self.len() as u8]).await?;
                for x in self.iter() {
                    <$el as $crate::WriteHci>::write_hci_async(x, &mut writer).await?;
                }
                Ok(())
            }
        }
    };
}

pub(crate) use param;