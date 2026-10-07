use crate::{EncodeError, Iap1Encode, Iap1EncodeCounted, Writer};
use catplay_reflect::{ProjectorContainerCapability, apply_container_erased, project_container_item};

struct Iap1EncodeContainer;

impl<T: Iap1Encode> ProjectorContainerCapability<T> for Iap1EncodeContainer {
    type Request = *mut Writer<'static>;
    type Error = EncodeError;

    unsafe fn project_item(item: *const T, (): &(), request: &mut Self::Request) -> Result<(), Self::Error> {
        // SAFETY: the shared container executor pairs this callback with a live T slice.
        let item = unsafe { &*item };
        // SAFETY: this raw writer pointer is created from the writer borrowed for encode_counted.
        let writer = unsafe { &mut **request };
        item.encode(writer)
    }
}

impl<T: Iap1Encode> Iap1EncodeCounted for [T] {
    fn encode_counted(&self, writer: &mut Writer<'_>, count: usize, field: &'static str) -> Result<(), EncodeError> {
        if self.len() != count {
            return Err(EncodeError::InvalidFieldValue { field });
        }

        let mut writer = (writer as *mut Writer<'_>).cast::<Writer<'static>>();
        // SAFETY: the slice pointer and length come from `self`, and the callback
        // uses the same T selected by this Iap1Encode implementation.
        unsafe {
            apply_container_erased(
                self.as_ptr().cast(),
                self.len(),
                &(),
                &mut writer,
                project_container_item::<T, _, Iap1EncodeContainer>,
            )
        }
    }
}
