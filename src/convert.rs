pub trait Serialize {
    fn serialize(self, buf: &mut [u8]);
}

macro_rules! impl_serialize {
    ($($ty:ty),*) => {
        $(impl Serialize for $ty {
            fn serialize(mut self, buf: &mut [u8]) {
                for byte in buf.iter_mut().rev() {
                    let windows_offset = 48;

                    if self == 0 {
                        *byte = windows_offset;
                    }

                    *byte = (self % 10) as u8 + windows_offset;
                    self /= 10;
                }
            }
        })*
    };

    ($ty1:ty as $ty2:ty) => {
        impl Serialize for $ty1 {
            fn serialize(self, buf: &mut [u8]) {
                let mut val = self as $ty2;
                for byte in buf.iter_mut().rev() {
                    let windows_offset = 48;

                    if val == 0 {
                        *byte = windows_offset;
                    }

                    let windows_offset = 48;
                    *byte = (val % 10) as u8 + windows_offset;
                    val /= 10;
                }
            }
        }
    }
}

impl_serialize!(u8, u16, u32, u64, u128, i8, i16, i32, i64, i128);
impl_serialize!(time::Month as u8);
