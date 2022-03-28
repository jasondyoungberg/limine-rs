#![no_std]

// misc
#[repr(transparent)]
pub struct LiminePtr<T>(*const T);

impl<T> LiminePtr<T> {
    const DEFAULT: LiminePtr<T> = Self(core::ptr::null_mut() as *const T);

    fn get(&self) -> *const T {
        self.0
    }
}

impl LiminePtr<char> {
    // todo: create a to_string() helper function to convert the null terminated
    // string to a rust string.
}

/// Used to create the limine request struct.
macro_rules! make_struct {
    (
        $(#[$meta:meta])*
        struct $name:ident: [$id1:expr, $id2:expr] => {
            $($(#[$field_meta:meta])* $field_name:ident : $field_ty:ty = $field_default:expr),*
        };
    ) => {
        $(#[$meta])*
        #[repr(C, packed)]
        pub struct $name {
            id: [u64; 4],
            revision: u64,

            $($field_name: $field_ty),*
        }

        impl $name {
            pub const ID: [u64; 4] = [0xc7b1dd30df4c8b88, 0x0a82e883a194f07b, $id1, $id2];

            pub const fn new(revision: u64) -> Self {
                Self {
                    id: Self::ID,
                    revision,

                    $($field_name: $field_default),*
                }
            }

            $($(#[$field_meta])* pub const fn $field_name(mut self, value: $field_ty) -> Self {
				self.$field_name = value;
				self
			})*
        }
    };
}

// boot info request tag:
pub struct LimineBootInfoResponse {
    pub revision: u64,

    pub name: LiminePtr<char>,
    pub version: LiminePtr<char>,
}

make_struct!(
    struct LimineBootInfoRequest: [0xf55038d8e2a1202f, 0x279426fcf5f59740] => {
        response: LiminePtr<LimineBootInfoResponse> = LiminePtr::DEFAULT
    };
);

// terminal request tag:
pub struct LimineTerminalResponse {
    pub revision: u64,

    pub columns: u32,
    pub rows: u32,

    write: LiminePtr<()>,
}

impl LimineTerminalResponse {
    pub fn write(&self) -> impl Fn(&str) {
        let __fn_ptr = self.write.get();
        let __term_func =
            unsafe { core::mem::transmute::<*const (), extern "C" fn(*const i8, u64)>(__fn_ptr) };

        move |txt| {
            __term_func(txt.as_ptr() as *const i8, txt.len() as u64);
        }
    }
}

make_struct!(
    struct LimineTerminalRequest: [0x0785a0aea5d0750f, 0x1c1936fee0d6cf6e] => {
        response: LiminePtr<LimineTerminalResponse> = LiminePtr::DEFAULT,
        callback: LiminePtr<()> = LiminePtr::DEFAULT
    };
);
