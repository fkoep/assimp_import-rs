macro_rules! ai_type {
    ($(#[$meta:meta])* type $name:ident: $raw_ty:ty;) => {

        #[repr(C)]
        $(#[$meta])*
        pub struct $name {
            raw: $raw_ty
        }

        impl $name { 
            #[doc(hidden)]
            pub unsafe fn slice<'a>(ptr: *mut $raw_ty, len: ::libc::c_uint) -> &'a [Self] {
                $crate::prim::slice::<$raw_ty, Self>(ptr, len)
            }
        }

    };
}

macro_rules! ai_ptr_type {
    ($(#[$meta:meta])* type $name:ident: $raw_ty:ty;) => {

        #[repr(C)]
        $(#[$meta])*
        pub struct $name<'a> {
            ptr: *mut $raw_ty,
            _p: ::std::marker::PhantomData<&'a ()>
        }

        impl<'a> $name<'a> {
            #[doc(hidden)]
            pub unsafe fn from_ptr(ptr: *mut $raw_ty) -> Self {
                assert!(!ptr.is_null());
                Self{ ptr: ptr, _p: ::std::marker::PhantomData }
            }

            #[doc(hidden)]
            pub unsafe fn slice(ptr: *mut*mut $raw_ty, len: ::libc::c_uint) -> &'a [Self] {
                $crate::prim::slice::<*mut $raw_ty, Self>(ptr, len)
            }

            #[doc(hidden)]
            pub fn raw(&self) -> &$raw_ty { unsafe { &*self.ptr } }

            #[doc(hidden)]
            // TODO Naming: get_ptr()
            pub fn as_ptr(&self) -> *mut $raw_ty { self.ptr }
        }

    };
}

/// TODO get rid of this, use FromPrimitive?
macro_rules! ai_impl_enum {
    ($ty:ty, $ffi_ty:ty) => {
        impl $ty {
            #[doc(hidden)]
            pub unsafe fn from_ffi(x: $ffi_ty) -> Self {
                ::std::mem::transmute(x)
            }
        }
    }
}
